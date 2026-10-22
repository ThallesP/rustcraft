#!/usr/bin/env python3
"""Check that a port of one of HotSpot's x86-64 libm stubs has the stub's instructions, in order.

Downloads the stub's C++ from the OpenJDK commit the JDK in vanilla/toolchain/ was built from
(its `release` file names it) into vanilla/hotspot/, and reads the port in
src/java/lang/math/hotspot_x86/ back into assembler calls (`xmm1.mulsd(xmm0);` is
`__ mulsd(xmm1, xmm0);`). Both must have the same instructions, and every straight-line run of
the C++ (between labels, jumps and compares) must appear unbroken in the port. Branch
conditions aren't checked here; the golden tests catch those.

    python3 tools/hotspot_port_check.py           # every ported stub
    python3 tools/hotspot_port_check.py sin cos   # just these
"""

import argparse
import re
import sys
import urllib.request
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
JDK_RELEASE = ROOT / "vanilla" / "toolchain" / "jdk-25" / "release"
PORT_DIR = ROOT / "src" / "java" / "lang" / "math" / "hotspot_x86"
SOURCE_URL = "https://raw.githubusercontent.com/adoptium/jdk25u/{commit}/src/hotspot/cpu/x86/{file}"

# Stubs that share a file with another, e.g. log10 lives in stubGenerator_x86_64_log.cpp.
FILES = {"log10": "log"}
# Instructions without an effect on the result that the port leaves out.
UNPORTED = {"bind", "jcc", "jmp", "cmpl", "cmpq", "testl", "ret", "leave", "enter", "push", "pop"}


def source(name):
    commit = re.search(r'^SOURCE=".*git:(\w+)"', JDK_RELEASE.read_text(), re.M).group(1)
    file = f"stubGenerator_x86_64_{FILES.get(name, name)}.cpp"
    path = ROOT / "vanilla" / "hotspot" / commit / file
    if not path.exists():
        path.parent.mkdir(parents=True, exist_ok=True)
        with urllib.request.urlopen(SOURCE_URL.format(commit=commit, file=file)) as response:
            path.write_bytes(response.read())
    return path.read_text()


def cpp_runs(text, name):
    start = text.index(f"address StubGenerator::generate_libm{name.capitalize()}()")
    body = text[start : text.index("return start;", start)]
    # Takes the SSE3 side of `if (VM_Version::supports_sse3())`: the other side does the same.
    body = re.sub(r"if \(VM_Version::supports_sse3\(\)\) \{\s*(__ [^;]*;)\s*\}\s*else\s*\{[^}]*\}", r"\1", body)
    runs, run = [], []
    for line in body.split("\n"):
        line = re.sub(r"//.*", "", line).strip()
        if not line.startswith("__ "):
            continue
        instruction = re.sub(r",\s*\w+\s*/\*rscratch\*/", "", line[3:].rstrip(";"))
        mnemonic = instruction.split("(")[0]
        stack = re.match(r"(subq|addq)\(rsp|mov(sd|q|l)\(Address\(rsp", instruction)
        if mnemonic in UNPORTED or stack:
            if mnemonic in ("bind", "jcc", "jmp") and run:
                runs.append(run)
                run = []
            continue
        run.append(normalize(instruction))
    return runs + [run] if run else runs


def rust_runs(text, name):
    start = text.index(f"pub fn libm_{name}(")
    end = text.find("pub fn libm_", start + 1)
    runs, run = [], []
    for line in text[start : end if end != -1 else None].split("\n"):
        lea = re.search(r"// (lea\(.*\))$", line)
        code = re.sub(r"//.*", "", line).strip()
        call = re.fullmatch(r"(?:let \w+ = )?(\w+)\.(\w+)\((.*)\);", code)
        if lea:
            run.append(normalize(lea.group(1)))
        elif call and not code.startswith("let rsp_"):
            run.append(normalize(rust_call(*call.groups())))
        elif not code.startswith("let rsp_"):
            if run:
                runs.append(run)
            run = []
    return runs + [run] if run else runs


def rust_call(register, mnemonic, args):
    args = re.sub(r"rsp_(\d+) >> 32", lambda m: f"Address(rsp, {int(m.group(1)) + 4})", args)
    args = re.sub(r"rsp_(\d+)", r"Address(rsp, \1)", args)
    args = args.replace(", borrow", "").replace("i32::MIN", "INT_MIN")
    args = re.sub(r"(-?\d+)i32 as u32", r"\1", args)

    def address(match):
        table, offset = match.groups()
        base = re.fullmatch(r"(r\w+)(?: \+ (\w+))?", offset)
        if base:
            return f"Address({base.group(1)}, {base.group(2) or 0})"
        return f"ExternalAddress({table}{'' if offset == '0' else ' + ' + offset})"

    args = re.sub(r"Address\(&(\w+), ([^)]*)\)", address, args)
    # Shifts by cl take no count operand in HotSpot.
    if mnemonic in ("shll", "shrl", "shlq", "shrq", "sarl") and args == "rcx":
        args = ""
    return f"{mnemonic}({register}{', ' + args if args else ''})"


# Table names differ in case between the two (Ctable, CTABLE).
def normalize(instruction):
    return instruction.replace(" ", "").lower()


def check(name):
    cpp = cpp_runs(source(name), name)
    rust = rust_runs((PORT_DIR / f"stub_generator_x86_64_{FILES.get(name, name)}.rs").read_text(), name)
    cpp_count = Counter(i for run in cpp for i in run)
    rust_count = Counter(i for run in rust for i in run)
    port = "".join(f"\n{instruction}" for run in rust for instruction in run) + "\n"
    broken = [run for run in cpp if "".join(f"\n{instruction}" for instruction in run) + "\n" not in port]
    print(f"{name}: {sum(cpp_count.values())} instructions in {len(cpp)} runs, port has {sum(rust_count.values())}")
    for instruction, count in (cpp_count - rust_count).items():
        print(f"  only in HotSpot: {instruction} x{count}")
    for instruction, count in (rust_count - cpp_count).items():
        print(f"  only in the port: {instruction} x{count}")
    for run in broken:
        print(f"  run not found in order: {run}")
    return cpp_count == rust_count and not broken


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("stubs", nargs="*", help="stub names, e.g. sin cos log (default: every ported one)")
    stubs = parser.parse_args().stubs or sorted(
        match for path in PORT_DIR.glob("*.rs") for match in re.findall(r"pub fn libm_(\w+)\(", path.read_text())
    )
    results = [check(name) for name in stubs]
    sys.exit(0 if all(results) else 1)


if __name__ == "__main__":
    main()
