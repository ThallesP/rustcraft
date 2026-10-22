//! net/minecraft/nbt/NbtAccounter.java
//!
//! Java throws `NbtAccounterException` (an `NbtException`, a `RuntimeException`) and
//! `IllegalArgumentException`, which are panics here.

use std::ops::{Deref, DerefMut};

/// Keeps a read of NBT within a quota of bytes, the ones the read tags take in Java's heap by its
/// own estimates, and within a depth of nested lists and compounds.
#[derive(Clone, Debug)]
pub struct NbtAccounter {
    quota: i64,
    usage: i64,
    max_depth: i32,
    depth: i32,
}

impl NbtAccounter {
    pub const DEFAULT_NBT_QUOTA: i32 = 2097152;
    pub const UNCOMPRESSED_NBT_QUOTA: i32 = 104857600;
    const MAX_STACK_DEPTH: i32 = 512;

    pub fn new(quota: i64, max_depth: i32) -> NbtAccounter {
        NbtAccounter {
            quota,
            usage: 0,
            max_depth,
            depth: 0,
        }
    }

    pub fn create(quota: i64) -> NbtAccounter {
        NbtAccounter::new(quota, NbtAccounter::MAX_STACK_DEPTH)
    }

    /// What a packet's NBT may take: 2 MiB.
    pub fn default_quota() -> NbtAccounter {
        NbtAccounter::new(2097152, NbtAccounter::MAX_STACK_DEPTH)
    }

    pub fn uncompressed_quota() -> NbtAccounter {
        NbtAccounter::new(104857600, NbtAccounter::MAX_STACK_DEPTH)
    }

    /// No quota to speak of, for what the client trusts (a block entity's data).
    pub fn unlimited_heap() -> NbtAccounter {
        NbtAccounter::new(i64::MAX, NbtAccounter::MAX_STACK_DEPTH)
    }

    /// `accountBytes(long, long)`: `count` entries of `bytes_per_entry`, a product that wraps.
    pub fn account_bytes_count(&mut self, bytes_per_entry: i64, count: i64) {
        self.account_bytes(bytes_per_entry.wrapping_mul(count));
    }

    /// Adds to the usage, panicking for a negative size or a usage past the quota. The sum
    /// wraps, so a usage near `i64::MAX` can go past it without panicking.
    pub fn account_bytes(&mut self, size: i64) {
        assert!(
            size >= 0,
            "Tried to account NBT tag with negative size: {size}"
        );
        assert!(
            self.usage.wrapping_add(size) <= self.quota,
            "Tried to read NBT tag that was too big; tried to allocate: {} + {size} bytes where max allowed: {}",
            self.usage,
            self.quota
        );
        self.usage = self.usage.wrapping_add(size);
    }

    pub fn push_depth(&mut self) {
        assert!(
            self.depth < self.max_depth,
            "Tried to read NBT tag with too high complexity, depth > {}",
            self.max_depth
        );
        self.depth += 1;
    }

    pub fn pop_depth(&mut self) {
        assert!(
            self.depth > 0,
            "NBT-Accounter tried to pop stack-depth at top-level"
        );
        self.depth -= 1;
    }

    /// `pushDepth()`, then `popDepth()` when the guard is dropped, which is Java's `finally`
    /// around a list's or compound's load and skip: one that panics still leaves the depth as it
    /// was. The guard derefs to the accounter for the load.
    pub(crate) fn push_depth_until_dropped(&mut self) -> PushedDepth<'_> {
        self.push_depth();
        PushedDepth { accounter: self }
    }

    pub fn get_usage(&self) -> i64 {
        self.usage
    }

    pub fn get_depth(&self) -> i32 {
        self.depth
    }
}

/// What `push_depth_until_dropped` hands out. `popDepth` can't panic in its drop, which would
/// abort a panicking thread: the depth it pops is the one it pushed.
pub(crate) struct PushedDepth<'a> {
    accounter: &'a mut NbtAccounter,
}

impl Deref for PushedDepth<'_> {
    type Target = NbtAccounter;

    fn deref(&self) -> &NbtAccounter {
        self.accounter
    }
}

impl DerefMut for PushedDepth<'_> {
    fn deref_mut(&mut self) -> &mut NbtAccounter {
        self.accounter
    }
}

impl Drop for PushedDepth<'_> {
    fn drop(&mut self) {
        self.accounter.pop_depth();
    }
}

#[cfg(test)]
mod tests {
    use super::NbtAccounter;
    use crate::golden::{Golden, encode_fields};

    // [quota, usage, maxDepth, depth], its fields.
    encode_fields!(NbtAccounter {
        quota: i64,
        usage: i64,
        max_depth: i32,
        depth: i32,
    });

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.nbt.NbtAccounter");
        golden.field("DEFAULT_NBT_QUOTA", NbtAccounter::DEFAULT_NBT_QUOTA);
        golden.field(
            "UNCOMPRESSED_NBT_QUOTA",
            NbtAccounter::UNCOMPRESSED_NBT_QUOTA,
        );
        golden.check("<init>(JI)V", NbtAccounter::new);
        golden.check(
            "create(J)Lnet/minecraft/nbt/NbtAccounter;",
            NbtAccounter::create,
        );
        golden.check(
            "defaultQuota()Lnet/minecraft/nbt/NbtAccounter;",
            NbtAccounter::default_quota,
        );
        golden.check(
            "uncompressedQuota()Lnet/minecraft/nbt/NbtAccounter;",
            NbtAccounter::uncompressed_quota,
        );
        golden.check(
            "unlimitedHeap()Lnet/minecraft/nbt/NbtAccounter;",
            NbtAccounter::unlimited_heap,
        );
        golden.check_method(
            "accountBytes(JJ)V",
            |this: &mut NbtAccounter, bytes_per_entry: i64, count: i64| {
                this.account_bytes_count(bytes_per_entry, count)
            },
        );
        golden.check_method("accountBytes(J)V", |this: &mut NbtAccounter, size: i64| {
            this.account_bytes(size)
        });
        golden.check_method("pushDepth()V", |this: &mut NbtAccounter| this.push_depth());
        golden.check_method("popDepth()V", |this: &mut NbtAccounter| this.pop_depth());
        golden.check_method("getUsage()J", |this: &mut NbtAccounter| this.get_usage());
        golden.check_method("getDepth()I", |this: &mut NbtAccounter| this.get_depth());
        golden.finish();
    }
}
