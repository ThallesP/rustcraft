use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde_json::Value;

use crate::value::{FloatBits, describe, flatten, ulps};

pub struct FieldDiff {
    pub field: String,
    pub a: Option<Value>,
    pub b: Option<Value>,
}

pub struct TraceDiff {
    pub ticks_a: usize,
    pub ticks_b: usize,
    pub diverging_ticks: usize,
    /// The first tick that differs, with every field that differs on it.
    pub first: Option<(String, Vec<FieldDiff>)>,
}

impl TraceDiff {
    pub fn is_same(&self) -> bool {
        self.diverging_ticks == 0
    }
}

/// Pairs ticks by their `tick` field, or by line when a trace has none, and compares every field.
pub fn compare(a: &[Value], b: &[Value]) -> TraceDiff {
    let use_tick_field = [a, b]
        .iter()
        .all(|trace| trace.iter().all(|line| line["tick"].is_i64()));
    let (ticks_a, ticks_b) = (by_tick(a, use_tick_field), by_tick(b, use_tick_field));
    let all_ticks: BTreeSet<i64> = ticks_a.keys().chain(ticks_b.keys()).copied().collect();

    let mut diverging_ticks = 0;
    let mut first = None;
    for tick in all_ticks {
        let fields = compare_tick(ticks_a.get(&tick).copied(), ticks_b.get(&tick).copied());
        if fields.is_empty() {
            continue;
        }
        diverging_ticks += 1;
        if first.is_none() {
            first = Some((tick.to_string(), fields));
        }
    }

    TraceDiff {
        ticks_a: a.len(),
        ticks_b: b.len(),
        diverging_ticks,
        first,
    }
}

fn by_tick(trace: &[Value], use_tick_field: bool) -> BTreeMap<i64, &Value> {
    trace
        .iter()
        .enumerate()
        .map(|(index, line)| {
            (
                if use_tick_field {
                    line["tick"].as_i64().unwrap()
                } else {
                    index as i64
                },
                line,
            )
        })
        .collect()
}

fn compare_tick(a: Option<&Value>, b: Option<&Value>) -> Vec<FieldDiff> {
    let (Some(a), Some(b)) = (a, b) else {
        let present = if a.is_some() {
            "only in A"
        } else {
            "only in B"
        };
        return vec![FieldDiff {
            field: format!("(tick {present})"),
            a: a.cloned(),
            b: b.cloned(),
        }];
    };

    let (a, b) = (flatten(a), flatten(b));
    let fields: BTreeSet<&String> = a.keys().chain(b.keys()).collect();
    fields
        .into_iter()
        .filter(|field| a.get(*field) != b.get(*field))
        .map(|field| FieldDiff {
            field: field.clone(),
            a: a.get(field).cloned(),
            b: b.get(field).cloned(),
        })
        .collect()
}

impl fmt::Display for TraceDiff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "trace: {} vs {} ticks", self.ticks_a, self.ticks_b)?;
        let Some((tick, fields)) = &self.first else {
            return writeln!(f, ", identical");
        };

        writeln!(
            f,
            ", {} differ; first at tick {tick}:",
            self.diverging_ticks
        )?;
        let width = fields
            .iter()
            .map(|diff| diff.field.len())
            .max()
            .unwrap_or(0);
        for diff in fields {
            write!(
                f,
                "  {:width$}  A = {}  B = {}",
                diff.field,
                describe(diff.a.as_ref()),
                describe(diff.b.as_ref())
            )?;
            let floats = diff
                .a
                .as_ref()
                .and_then(FloatBits::parse)
                .zip(diff.b.as_ref().and_then(FloatBits::parse));
            if let Some(distance) = floats.and_then(|(a, b)| ulps(a, b)) {
                write!(f, "  [{distance} ulp]")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn tick(tick: i64, y: f64, on_ground: bool) -> Value {
        json!({ "tick": tick, "pos": ["0000000000000000", format!("{:016x}", y.to_bits())], "on_ground": on_ground })
    }

    #[test]
    fn identical_traces_match() {
        let trace = [tick(0, -60.0, true), tick(1, -60.0, true)];
        let diff = compare(&trace, &trace);
        assert!(diff.is_same());
        assert!(diff.to_string().contains("identical"));
    }

    #[test]
    fn reports_the_first_diverging_tick_and_its_fields() {
        let a = [
            tick(0, -60.0, true),
            tick(1, -59.5, false),
            tick(2, -59.0, false),
        ];
        let b = [
            tick(0, -60.0, true),
            tick(1, (-59.5f64).next_up(), true),
            tick(2, -59.0, false),
        ];
        let diff = compare(&a, &b);

        assert_eq!(diff.diverging_ticks, 1);
        let (first_tick, fields) = diff.first.as_ref().unwrap();
        assert_eq!(first_tick, "1");
        let names: Vec<&str> = fields.iter().map(|diff| diff.field.as_str()).collect();
        assert_eq!(names, ["on_ground", "pos[1]"]);
        assert!(diff.to_string().contains("[1 ulp]"));
    }

    #[test]
    fn missing_ticks_count_as_divergence() {
        let a = [tick(0, -60.0, true), tick(1, -60.0, true)];
        let diff = compare(&a, &a[..1]);
        assert_eq!(diff.diverging_ticks, 1);
        assert!(diff.to_string().contains("only in A"));
    }
}
