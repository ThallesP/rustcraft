use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde_json::Value;

/// Packets that differ between two runs of the same flow against the same vanilla server, so a
/// diff skips them entirely (seen by capturing one flow twice against fresh servers):
/// - `keep_alive`, `ping`, `pong`: random ids, sent on a wall-clock timer.
/// - `ping_request`, `pong_response`: the status ping carries the client's clock.
/// - `login_finished`: carries a random session id next to the profile.
/// - `set_time`: sent on a timer, with a game time that depends on how long the server ran.
/// - `update_advancements`: carries wall-clock timestamps of obtained criteria.
/// - `chunk_batch_*`: chunk pacing. The client answers each batch with a rate it measured with
///   its clock, and the server sizes the next batches from it.
pub const DEFAULT_IGNORED: &[&str] = &[
    "minecraft:keep_alive",
    "minecraft:ping",
    "minecraft:pong",
    "minecraft:ping_request",
    "minecraft:pong_response",
    "minecraft:login_finished",
    "minecraft:set_time",
    "minecraft:update_advancements",
    "minecraft:chunk_batch_start",
    "minecraft:chunk_batch_finished",
    "minecraft:chunk_batch_received",
];

/// What clients send right before scenario tick 0: a `custom_payload` with the identifier
/// `rustcraft:scenario_start` and no data.
const SCENARIO_START_NAME: &str = "minecraft:custom_payload";
const SCENARIO_START_PAYLOAD: &[u8] = b"\x18rustcraft:scenario_start";

/// Serverbound packets a client sends every tick, or every few ticks, whatever it is doing.
/// While joining, the client is stepped one tick at a time and the server runs in real time, so
/// how many of these the join takes, and where they fall, is timing. The join phase skips them.
pub const JOIN_PER_TICK: &[&str] = &[
    "minecraft:client_tick_end",
    "minecraft:move_player_pos",
    "minecraft:move_player_pos_rot",
    "minecraft:move_player_rot",
    "minecraft:move_player_status_only",
    "minecraft:player_input",
];

pub struct Packet {
    pub connection: i64,
    pub direction: String,
    pub state: String,
    pub name: String,
    pub payload: Vec<u8>,
}

impl Packet {
    pub fn parse(line: &Value) -> Result<Self, String> {
        let text = |field: &str| {
            line[field]
                .as_str()
                .map(str::to_string)
                .ok_or(format!("missing {field}"))
        };
        let name = match &line["name"] {
            Value::String(name) => name.clone(),
            _ => format!("id:{}", line["id"]),
        };
        Ok(Self {
            connection: line["connection"].as_i64().unwrap_or(0),
            direction: text("direction")?,
            state: text("state")?,
            name,
            payload: decode_hex(&text("payload")?)?,
        })
    }

    fn label(&self) -> String {
        format!("{} {}", self.state, self.name)
    }

    fn is_scenario_start(&self) -> bool {
        self.name == SCENARIO_START_NAME && self.payload == SCENARIO_START_PAYLOAD
    }

    fn exact_payload(&self) -> Vec<u8> {
        self.payload.clone()
    }

    // Equal for two payloads holding the same bytes in any order.
    fn sorted_payload(&self) -> Vec<u8> {
        let mut bytes = self.payload.clone();
        bytes.sort_unstable();
        bytes
    }
}

fn decode_hex(hex: &str) -> Result<Vec<u8>, String> {
    if !hex.len().is_multiple_of(2) {
        return Err(format!(
            "payload has an odd number of hex digits: {}",
            hex.len()
        ));
    }
    (0..hex.len())
        .step_by(2)
        .map(|offset| {
            u8::from_str_radix(&hex[offset..offset + 2], 16)
                .map_err(|_| format!("payload isn't hex at offset {offset}"))
        })
        .collect()
}

pub struct PayloadDiff {
    pub index: usize,
    pub packet: String,
    pub length_a: usize,
    pub length_b: usize,
    pub differing_bytes: usize,
    pub first_offset: usize,
}

/// Where an ordered stream splits: index, packet in A, packet in B (`None` once a stream ended).
pub type SequenceSplit = (usize, Option<String>, Option<String>);

pub enum Comparison {
    /// In order, byte for byte.
    Ordered {
        /// The first position where the sequence differs.
        structural: Option<SequenceSplit>,
        /// Payload differences among the packets before the structural difference.
        payloads: Vec<PayloadDiff>,
    },
    /// In any order: packets one side has more of, as (packet, bytes, count in A minus count in B).
    Unordered {
        unmatched: Vec<(String, usize, i64)>,
    },
}

impl Comparison {
    fn is_same(&self) -> bool {
        match self {
            Comparison::Ordered {
                structural,
                payloads,
            } => structural.is_none() && payloads.is_empty(),
            Comparison::Unordered { unmatched } => unmatched.is_empty(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase {
    /// The whole stream, when there's no scenario-start marker to split it at.
    Whole,
    /// Serverbound packets before the scenario-start marker: joining and scenario setup.
    Join,
    /// Serverbound packets from the scenario-start marker on, the marker being #0.
    Scenario,
}

pub struct PhaseDiff {
    pub phase: Phase,
    pub compared: (usize, usize),
    /// Join phase: per-tick packets skipped on each side.
    pub per_tick_skipped: (usize, usize),
    pub comparison: Comparison,
}

pub struct StreamDiff {
    pub connection: i64,
    pub direction: String,
    pub packets: (usize, usize),
    pub ignored: (usize, usize),
    /// Set when only one capture has the scenario-start marker: "A" or "B".
    pub marker_only_in: Option<&'static str>,
    pub phases: Vec<PhaseDiff>,
}

impl StreamDiff {
    fn is_same(&self) -> bool {
        self.marker_only_in.is_none() && self.phases.iter().all(|phase| phase.comparison.is_same())
    }
}

pub struct CaptureDiff {
    pub ignored: Vec<String>,
    /// Only the join phase was compared (`--join`).
    pub join_only: bool,
    pub streams: Vec<StreamDiff>,
}

impl CaptureDiff {
    pub fn is_same(&self) -> bool {
        self.streams.iter().all(StreamDiff::is_same)
    }
}

/// Compares each connection and direction on its own, since how the two directions interleave
/// depends on timing.
///
/// Serverbound packets are the client's behaviour. When both captures have the scenario-start
/// marker on a connection, the packets from it on must match in order, byte for byte, and the
/// join before it (where the client is stepped while the server runs in real time) only needs the
/// same packets with the same bytes in any order, per-tick packets aside. Without a marker the
/// whole serverbound stream must match in order. A marker on one side only is a difference.
///
/// Clientbound packets come from the vanilla server, which doesn't repeat itself exactly: tick
/// timing moves packets around, and collections it writes from Java `HashMap`s (tags, recipes,
/// commands, chunk heightmaps) come out in a different order every JVM run. So unless
/// `ordered_clientbound` is set, clientbound streams only need the same packets with the same
/// bytes, in any order. A server reacting differently (a correction teleport, a missing chunk)
/// still shows up as a packet only one side got.
#[cfg(test)]
pub fn compare(
    a: &[Packet],
    b: &[Packet],
    ignored: &[String],
    ordered_clientbound: bool,
) -> CaptureDiff {
    compare_scoped(a, b, ignored, ordered_clientbound, false)
}

/// `compare`, or with `join_only` only the join phase: on a connection whose captures both have
/// the scenario-start marker, the serverbound packets before it, compared as the join phase
/// always is, and the clientbound packets that came before the client sent it. What a client
/// does from the marker on is the scenario's, left out. A connection with no marker on either
/// side is compared whole, as it always is, and one with a marker on one side only still differs.
pub fn compare_scoped(
    a: &[Packet],
    b: &[Packet],
    ignored: &[String],
    ordered_clientbound: bool,
    join_only: bool,
) -> CaptureDiff {
    let streams: BTreeSet<(i64, &str)> = a
        .iter()
        .chain(b)
        .map(|packet| (packet.connection, packet.direction.as_str()))
        .collect();
    let streams = streams
        .into_iter()
        .map(|(connection, direction)| {
            let (mut stream_a, mut stream_b) = (
                select(a, connection, direction),
                select(b, connection, direction),
            );
            if join_only && direction == "clientbound" {
                let (marker_a, marker_b) =
                    (marker_index(a, connection), marker_index(b, connection));
                if let (Some(marker_a), Some(marker_b)) = (marker_a, marker_b) {
                    stream_a = select(&a[..marker_a], connection, direction);
                    stream_b = select(&b[..marker_b], connection, direction);
                }
            }
            compare_stream(
                connection,
                direction,
                &stream_a,
                &stream_b,
                ignored,
                ordered_clientbound,
                join_only,
            )
        })
        .collect();
    CaptureDiff {
        ignored: ignored.to_vec(),
        join_only,
        streams,
    }
}

/// Where in a capture the connection's client sent the scenario-start marker.
fn marker_index(capture: &[Packet], connection: i64) -> Option<usize> {
    capture.iter().position(|packet| {
        packet.connection == connection
            && packet.direction == "serverbound"
            && packet.is_scenario_start()
    })
}

fn compare_stream(
    connection: i64,
    direction: &str,
    a: &[&Packet],
    b: &[&Packet],
    ignored: &[String],
    ordered_clientbound: bool,
    join_only: bool,
) -> StreamDiff {
    let (kept_a, kept_b) = (kept(a, ignored), kept(b, ignored));
    let mut diff = StreamDiff {
        connection,
        direction: direction.to_string(),
        packets: (a.len(), b.len()),
        ignored: (a.len() - kept_a.len(), b.len() - kept_b.len()),
        marker_only_in: None,
        phases: Vec::new(),
    };

    if direction == "clientbound" && !ordered_clientbound {
        diff.phases.push(whole(
            &kept_a,
            &kept_b,
            compare_unordered(&kept_a, &kept_b, Packet::sorted_payload),
        ));
        return diff;
    }

    let marker = |stream: &[&Packet]| {
        let serverbound = direction == "serverbound";
        stream
            .iter()
            .position(|packet| serverbound && packet.is_scenario_start())
    };
    match (marker(a), marker(b)) {
        (None, None) => {
            diff.phases
                .push(whole(&kept_a, &kept_b, compare_ordered(&kept_a, &kept_b)))
        }
        (Some(marker_a), Some(marker_b)) => {
            diff.phases
                .push(compare_join(&a[..marker_a], &b[..marker_b], ignored));
            if join_only {
                return diff;
            }
            let (scenario_a, scenario_b) =
                (kept(&a[marker_a..], ignored), kept(&b[marker_b..], ignored));
            diff.phases.push(PhaseDiff {
                phase: Phase::Scenario,
                compared: (scenario_a.len(), scenario_b.len()),
                per_tick_skipped: (0, 0),
                comparison: compare_ordered(&scenario_a, &scenario_b),
            });
        }
        (Some(_), None) => diff.marker_only_in = Some("A"),
        (None, Some(_)) => diff.marker_only_in = Some("B"),
    }
    diff
}

fn whole(a: &[&Packet], b: &[&Packet], comparison: Comparison) -> PhaseDiff {
    PhaseDiff {
        phase: Phase::Whole,
        compared: (a.len(), b.len()),
        per_tick_skipped: (0, 0),
        comparison,
    }
}

fn compare_join(a: &[&Packet], b: &[&Packet], ignored: &[String]) -> PhaseDiff {
    let ((compared_a, skipped_a), (compared_b, skipped_b)) =
        (without_per_tick(a, ignored), without_per_tick(b, ignored));

    PhaseDiff {
        phase: Phase::Join,
        compared: (compared_a.len(), compared_b.len()),
        per_tick_skipped: (skipped_a, skipped_b),
        comparison: compare_unordered(&compared_a, &compared_b, Packet::exact_payload),
    }
}

// The packets left after the ignore list and the per-tick packets, and how many per-tick ones went.
fn without_per_tick<'a>(stream: &[&'a Packet], ignored: &[String]) -> (Vec<&'a Packet>, usize) {
    let kept_stream = kept(stream, ignored);
    let compared: Vec<&Packet> = kept_stream
        .iter()
        .copied()
        .filter(|packet| !JOIN_PER_TICK.contains(&packet.name.as_str()))
        .collect();
    let skipped = kept_stream.len() - compared.len();
    (compared, skipped)
}

fn select<'a>(capture: &'a [Packet], connection: i64, direction: &str) -> Vec<&'a Packet> {
    capture
        .iter()
        .filter(|packet| packet.connection == connection && packet.direction == direction)
        .collect()
}

fn kept<'a>(stream: &[&'a Packet], ignored: &[String]) -> Vec<&'a Packet> {
    stream
        .iter()
        .copied()
        .filter(|packet| !ignored.contains(&packet.name))
        .collect()
}

fn compare_ordered(a: &[&Packet], b: &[&Packet]) -> Comparison {
    let structural = (0..a.len().max(b.len())).find_map(|index| {
        let (packet_a, packet_b) = (
            a.get(index).map(|packet| packet.label()),
            b.get(index).map(|packet| packet.label()),
        );
        (packet_a != packet_b).then_some((index, packet_a, packet_b))
    });

    let comparable = structural.as_ref().map_or(a.len(), |(index, _, _)| *index);
    let payloads = (0..comparable)
        .filter_map(|index| compare_payload(index, a[index], b[index]))
        .collect();
    Comparison::Ordered {
        structural,
        payloads,
    }
}

fn compare_payload(index: usize, a: &Packet, b: &Packet) -> Option<PayloadDiff> {
    if a.payload == b.payload {
        return None;
    }

    let differing: Vec<usize> = a
        .payload
        .iter()
        .zip(&b.payload)
        .enumerate()
        .filter(|(_, (x, y))| x != y)
        .map(|(offset, _)| offset)
        .collect();
    let (length_a, length_b) = (a.payload.len(), b.payload.len());

    Some(PayloadDiff {
        index,
        packet: a.label(),
        length_a,
        length_b,
        differing_bytes: differing.len() + length_a.abs_diff(length_b),
        first_offset: differing.first().copied().unwrap_or(length_a.min(length_b)),
    })
}

/// Counts packets by name and `canonical` payload; whatever doesn't cancel out is unmatched.
fn compare_unordered(
    a: &[&Packet],
    b: &[&Packet],
    canonical: fn(&Packet) -> Vec<u8>,
) -> Comparison {
    let mut surplus: BTreeMap<(String, Vec<u8>), i64> = BTreeMap::new();
    for (stream, sign) in [(a, 1), (b, -1)] {
        for packet in stream {
            *surplus
                .entry((packet.label(), canonical(packet)))
                .or_default() += sign;
        }
    }
    let unmatched = surplus
        .into_iter()
        .filter(|(_, count)| *count != 0)
        .map(|((label, bytes), count)| (label, bytes.len(), count))
        .collect();
    Comparison::Unordered { unmatched }
}

const MAX_LISTED: usize = 20;

impl fmt::Display for CaptureDiff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "packets ({}ignoring {}):",
            if self.join_only {
                "join phase only, "
            } else {
                ""
            },
            if self.ignored.is_empty() {
                "nothing".to_string()
            } else {
                self.ignored.join(", ")
            }
        )?;
        for stream in &self.streams {
            write!(
                f,
                "  connection {} {}: {} vs {} packets, {} vs {} ignored",
                stream.connection,
                stream.direction,
                stream.packets.0,
                stream.packets.1,
                stream.ignored.0,
                stream.ignored.1
            )?;
            if let Some(side) = stream.marker_only_in {
                writeln!(
                    f,
                    "\n    scenario-start marker only in {side}: the captures disagree on where the scenario starts"
                )?;
                continue;
            }

            match stream.phases.as_slice() {
                [whole] if whole.phase == Phase::Whole => {
                    if whole.comparison.is_same() {
                        let same = match whole.comparison {
                            Comparison::Ordered { .. } => "identical",
                            Comparison::Unordered { .. } => "same packets, order aside",
                        };
                        writeln!(f, ", {same}")?;
                    } else {
                        writeln!(f)?;
                        write_comparison(f, whole)?;
                    }
                }
                phases => {
                    writeln!(f)?;
                    for phase in phases {
                        write_phase_summary(f, phase)?;
                        write_comparison(f, phase)?;
                    }
                }
            }
        }
        Ok(())
    }
}

fn write_phase_summary(f: &mut fmt::Formatter<'_>, phase: &PhaseDiff) -> fmt::Result {
    let (a, b) = phase.compared;
    let verdict = if phase.comparison.is_same() {
        match phase.comparison {
            Comparison::Ordered { .. } => "identical",
            Comparison::Unordered { .. } => "same packets, order aside",
        }
    } else {
        "differs"
    };
    match phase.phase {
        Phase::Join => {
            let (skipped_a, skipped_b) = phase.per_tick_skipped;
            writeln!(
                f,
                "    join phase: {a} vs {b} packets before the scenario-start marker ({skipped_a} vs {skipped_b} per-tick skipped), {verdict}"
            )
        }
        Phase::Scenario => writeln!(
            f,
            "    scenario: {a} vs {b} packets from the marker on, {verdict}"
        ),
        Phase::Whole => Ok(()),
    }
}

fn write_comparison(f: &mut fmt::Formatter<'_>, phase: &PhaseDiff) -> fmt::Result {
    let scenario = phase.phase == Phase::Scenario;
    let sequence_at = |index: usize| match scenario {
        true => format!("scenario: sequence differs at packet #{index} after the marker"),
        false => format!("sequence differs at #{index}"),
    };
    let payload_at = |index: usize, packet: &str| match scenario {
        true => format!("scenario, packet #{index} after the marker ({packet})"),
        false => format!("#{index} {packet}"),
    };
    let unordered_prefix = match phase.phase {
        Phase::Join => "join phase, ",
        _ => "",
    };

    match &phase.comparison {
        Comparison::Ordered {
            structural,
            payloads,
        } => {
            if let Some((index, a, b)) = structural {
                let missing = || "(stream ended)".to_string();
                writeln!(f, "    {}:", sequence_at(*index))?;
                writeln!(f, "      A: {}", a.clone().unwrap_or_else(missing))?;
                writeln!(f, "      B: {}", b.clone().unwrap_or_else(missing))?;
            }
            for payload in payloads.iter().take(MAX_LISTED) {
                writeln!(
                    f,
                    "    {}: {} vs {} bytes, {} differ from byte {}",
                    payload_at(payload.index, &payload.packet),
                    payload.length_a,
                    payload.length_b,
                    payload.differing_bytes,
                    payload.first_offset
                )?;
            }
            if payloads.len() > MAX_LISTED {
                writeln!(f, "    … and {} more", payloads.len() - MAX_LISTED)?;
            }
        }
        Comparison::Unordered { unmatched } => {
            for (label, bytes, count) in unmatched.iter().take(MAX_LISTED) {
                let side = if *count > 0 { "A" } else { "B" };
                writeln!(
                    f,
                    "    {unordered_prefix}only in {side}: {}x {label} ({bytes} bytes)",
                    count.abs()
                )?;
            }
            if unmatched.len() > MAX_LISTED {
                writeln!(f, "    … and {} more", unmatched.len() - MAX_LISTED)?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn packet(direction: &str, state: &str, name: &str, payload: &str) -> Packet {
        Packet {
            connection: 0,
            direction: direction.into(),
            state: state.into(),
            name: name.into(),
            payload: decode_hex(payload).unwrap(),
        }
    }

    fn serverbound(name: &str, payload: &str) -> Packet {
        packet("serverbound", "play", &format!("minecraft:{name}"), payload)
    }

    fn marker() -> Packet {
        Packet {
            payload: SCENARIO_START_PAYLOAD.to_vec(),
            ..serverbound("custom_payload", "")
        }
    }

    fn default_ignored() -> Vec<String> {
        DEFAULT_IGNORED
            .iter()
            .map(|name| name.to_string())
            .collect()
    }

    fn ordered(phase: &PhaseDiff) -> (&Option<SequenceSplit>, &Vec<PayloadDiff>) {
        match &phase.comparison {
            Comparison::Ordered {
                structural,
                payloads,
            } => (structural, payloads),
            Comparison::Unordered { .. } => panic!("expected an ordered comparison"),
        }
    }

    fn unmatched(phase: &PhaseDiff) -> &Vec<(String, usize, i64)> {
        match &phase.comparison {
            Comparison::Unordered { unmatched } => unmatched,
            Comparison::Ordered { .. } => panic!("expected an unordered comparison"),
        }
    }

    // The join of the real `walk` determinism fixture, shortened: the two runs took a different
    // number of ticks before `player_loaded`.
    fn walk_run(ticks_before_loaded: usize, scenario: &[Packet]) -> Vec<Packet> {
        let mut run = vec![
            serverbound("client_information", "05656e5f7573"),
            serverbound("accept_teleportation", "013fe0"),
        ];
        run.extend((0..ticks_before_loaded).map(|_| serverbound("client_tick_end", "")));
        run.extend([
            serverbound("player_loaded", ""),
            serverbound("move_player_pos", "3fe0"),
            serverbound("chunk_batch_received", "42800000"),
            serverbound("chat_command", "0d74696d65"),
            serverbound("client_tick_end", ""),
            marker(),
        ]);
        run.extend(scenario.iter().map(|packet| Packet {
            name: packet.name.clone(),
            payload: packet.payload.clone(),
            ..serverbound("", "")
        }));
        run
    }

    fn walk_scenario() -> Vec<Packet> {
        vec![
            serverbound("player_input", "01"),
            serverbound("move_player_pos", "3fe1"),
            serverbound("client_tick_end", ""),
            serverbound("move_player_pos", "3fe2"),
            serverbound("client_tick_end", ""),
        ]
    }

    #[test]
    fn join_only_leaves_the_scenario_out() {
        let a = walk_run(3, &walk_scenario());
        let b = walk_run(5, &walk_scenario()[..2]);
        assert!(!compare(&a, &b, &default_ignored(), false).is_same());

        let diff = compare_scoped(&a, &b, &default_ignored(), false, true);
        assert!(diff.is_same(), "{diff}");
        assert_eq!(diff.streams[0].phases.len(), 1);
        assert_eq!(diff.streams[0].phases[0].phase, Phase::Join);
        assert!(diff.to_string().contains("join phase only"));
    }

    #[test]
    fn join_only_still_catches_a_changed_join() {
        let a = walk_run(3, &walk_scenario());
        let mut b = walk_run(3, &walk_scenario());
        b[0] = serverbound("client_information", "05656e5f7573ff");
        let diff = compare_scoped(&a, &b, &default_ignored(), false, true);
        assert!(!diff.is_same());
    }

    #[test]
    fn join_only_cuts_clientbound_packets_at_the_marker() {
        let clientbound = |name: &str, payload: &str| packet("clientbound", "play", name, payload);
        let mut a = walk_run(3, &[]);
        a.insert(1, clientbound("minecraft:login", "01"));
        a.push(clientbound("minecraft:set_entity_data", "0100"));
        let mut b = walk_run(4, &[]);
        b.insert(2, clientbound("minecraft:login", "01"));
        b.push(clientbound("minecraft:set_entity_data", "0101"));

        assert!(!compare(&a, &b, &default_ignored(), false).is_same());
        let diff = compare_scoped(&a, &b, &default_ignored(), false, true);
        assert!(diff.is_same(), "{diff}");

        b.insert(3, clientbound("minecraft:game_event", "0d"));
        assert!(!compare_scoped(&a, &b, &default_ignored(), false, true).is_same());
    }

    #[test]
    fn same_flow_with_different_keep_alives_matches() {
        let a = [
            serverbound("move_player_pos", "0102"),
            serverbound("keep_alive", "aaaa"),
            serverbound("client_tick_end", ""),
        ];
        let b = [
            serverbound("move_player_pos", "0102"),
            serverbound("client_tick_end", ""),
        ];
        let diff = compare(&a, &b, &default_ignored(), false);
        assert!(diff.is_same(), "{diff}");
    }

    #[test]
    fn reports_serverbound_payload_bytes_that_differ() {
        let a = [serverbound("move_player_pos", "00112233")];
        let b = [serverbound("move_player_pos", "0011ff33ee")];
        let diff = compare(&a, &b, &[], false);
        let (_, payloads) = ordered(&diff.streams[0].phases[0]);
        let payload = &payloads[0];
        assert_eq!(
            (
                payload.length_a,
                payload.length_b,
                payload.differing_bytes,
                payload.first_offset
            ),
            (4, 5, 2, 2)
        );
    }

    #[test]
    fn serverbound_reorderings_are_differences() {
        let a = [serverbound("move_player_pos", "0102")];
        let b = [serverbound("move_player_pos", "0201")];
        assert!(!compare(&a, &b, &[], false).is_same());
    }

    #[test]
    fn reports_where_the_sequence_splits() {
        let a = [serverbound("chat", ""), serverbound("swing", "")];
        let b = [serverbound("chat", ""), serverbound("use_item", "")];
        let diff = compare(&a, &b, &[], false);
        let (structural, _) = ordered(&diff.streams[0].phases[0]);
        let (index, packet_a, packet_b) = structural.clone().unwrap();
        assert_eq!(index, 1);
        assert_eq!(packet_a.as_deref(), Some("play minecraft:swing"));
        assert_eq!(packet_b.as_deref(), Some("play minecraft:use_item"));
    }

    #[test]
    fn without_markers_the_join_timing_is_a_difference() {
        let (a, b) = (walk_run(5, &walk_scenario()), walk_run(6, &walk_scenario()));
        let unmarked = |run: Vec<Packet>| -> Vec<Packet> {
            run.into_iter()
                .filter(|packet| !packet.is_scenario_start())
                .collect()
        };
        let diff = compare(&unmarked(a), &unmarked(b), &default_ignored(), false);
        assert!(!diff.is_same());
        assert_eq!(diff.streams[0].phases[0].phase, Phase::Whole);
    }

    #[test]
    fn join_timing_before_the_marker_is_tolerated() {
        let (a, b) = (walk_run(5, &walk_scenario()), walk_run(6, &walk_scenario()));
        let diff = compare(&a, &b, &default_ignored(), false);
        assert!(diff.is_same(), "{diff}");

        let phases = &diff.streams[0].phases;
        assert_eq!(
            (phases[0].phase, phases[1].phase),
            (Phase::Join, Phase::Scenario)
        );
        assert_eq!(phases[0].per_tick_skipped, (7, 8));
        assert_eq!(phases[1].compared, (6, 6));
        assert!(diff.to_string().contains(
            "join phase: 4 vs 4 packets before the scenario-start marker (7 vs 8 per-tick skipped), same packets, order aside"
        ));
    }

    #[test]
    fn join_phase_catches_missing_and_changed_setup_packets() {
        let a = walk_run(5, &walk_scenario());
        let mut b = walk_run(6, &walk_scenario());
        b.retain(|packet| packet.name != "minecraft:client_information");
        let accept_teleportation = b
            .iter_mut()
            .find(|packet| packet.name == "minecraft:accept_teleportation")
            .unwrap();
        accept_teleportation.payload = vec![0x01, 0x3f, 0xe1];

        let diff = compare(&a, &b, &default_ignored(), false);
        assert!(!diff.is_same());
        let join = &diff.streams[0].phases[0];
        assert_eq!(
            unmatched(join),
            &[
                ("play minecraft:accept_teleportation".to_string(), 3, 1),
                ("play minecraft:accept_teleportation".to_string(), 3, -1),
                ("play minecraft:client_information".to_string(), 6, 1),
            ]
        );
        assert!(diff.streams[0].phases[1].comparison.is_same());
        assert!(
            diff.to_string()
                .contains("join phase, only in A: 1x play minecraft:client_information")
        );
    }

    #[test]
    fn scenario_after_the_marker_is_strict() {
        let mut changed = walk_scenario();
        changed[3].payload = vec![0x3f, 0xe3];
        let (a, b) = (walk_run(5, &walk_scenario()), walk_run(5, &changed));

        let diff = compare(&a, &b, &default_ignored(), false);
        let scenario = &diff.streams[0].phases[1];
        let (structural, payloads) = ordered(scenario);
        assert!(structural.is_none());
        assert_eq!(payloads[0].index, 4);
        assert!(diff.to_string().contains(
            "scenario, packet #4 after the marker (play minecraft:move_player_pos): 2 vs 2 bytes, 1 differ from byte 1"
        ));
    }

    #[test]
    fn scenario_order_after_the_marker_counts() {
        let mut swapped = walk_scenario();
        swapped.swap(1, 2);
        let diff = compare(
            &walk_run(5, &walk_scenario()),
            &walk_run(5, &swapped),
            &default_ignored(),
            false,
        );
        let (structural, _) = ordered(&diff.streams[0].phases[1]);
        assert_eq!(structural.as_ref().unwrap().0, 2);
        assert!(
            diff.to_string()
                .contains("scenario: sequence differs at packet #2 after the marker")
        );
    }

    #[test]
    fn a_marker_on_one_side_only_is_a_difference() {
        let a = walk_run(5, &walk_scenario());
        let b: Vec<Packet> = walk_run(5, &walk_scenario())
            .into_iter()
            .filter(|packet| !packet.is_scenario_start())
            .collect();
        let diff = compare(&a, &b, &default_ignored(), false);
        assert!(!diff.is_same());
        assert_eq!(diff.streams[0].marker_only_in, Some("A"));
        assert!(diff.to_string().contains("scenario-start marker only in A"));
    }

    #[test]
    fn marker_needs_the_exact_identifier_and_no_data() {
        let other = serverbound("custom_payload", "126d696e6563726166743a7265676973746572");
        let with_data = Packet {
            payload: [SCENARIO_START_PAYLOAD, &[0]].concat(),
            ..serverbound("custom_payload", "")
        };
        assert!(marker().is_scenario_start());
        assert!(!other.is_scenario_start());
        assert!(!with_data.is_scenario_start());
    }

    #[test]
    fn clientbound_order_and_hash_map_order_are_tolerated() {
        let a = [
            packet(
                "clientbound",
                "play",
                "minecraft:level_chunk_with_light",
                "0a0b0c",
            ),
            packet(
                "clientbound",
                "play",
                "minecraft:chunk_batch_finished",
                "01",
            ),
        ];
        let b = [
            packet(
                "clientbound",
                "play",
                "minecraft:chunk_batch_finished",
                "01",
            ),
            packet(
                "clientbound",
                "play",
                "minecraft:level_chunk_with_light",
                "0c0a0b",
            ),
        ];
        assert!(compare(&a, &b, &[], false).is_same());
        assert!(!compare(&a, &b, &[], true).is_same());
    }

    #[test]
    fn clientbound_reports_packets_only_one_side_got() {
        let a = [packet("clientbound", "play", "minecraft:login", "00")];
        let b = [
            packet("clientbound", "play", "minecraft:login", "00"),
            packet("clientbound", "play", "minecraft:player_position", "0102"),
        ];
        let diff = compare(&a, &b, &[], false);
        assert_eq!(
            unmatched(&diff.streams[0].phases[0]),
            &[("play minecraft:player_position".to_string(), 2, -1)]
        );
        assert!(
            diff.to_string()
                .contains("only in B: 1x play minecraft:player_position")
        );
    }

    #[test]
    fn directions_are_compared_separately() {
        let a = [
            packet("serverbound", "play", "minecraft:a", ""),
            packet("clientbound", "play", "minecraft:b", ""),
        ];
        let b = [
            packet("clientbound", "play", "minecraft:b", ""),
            packet("serverbound", "play", "minecraft:a", ""),
        ];
        assert!(compare(&a, &b, &[], true).is_same());
    }
}
