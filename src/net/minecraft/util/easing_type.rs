//! net/minecraft/util/EasingType.java
//!
//! How a keyframe track eases between keyframes. Only the curves the vanilla pack's timelines use
//! are ported: `constant`, `linear` and cubic Béziers. The named curves of `Ease` (`in_back`,
//! `out_bounce`, …) aren't ported, and `CODEC` reading one of their names is an error. `CODEC` is
//! `from_json`, decoded by hand like DataFixerUpper's `either` of a name and a cubic Bézier.

use crate::com::google::gson::json_element::JsonElement;
use crate::net::minecraft::util::mth::Mth;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EasingType {
    CONSTANT,
    LINEAR,
    CubicBezier(CubicBezier),
}

impl EasingType {
    pub fn cubic_bezier(x1: f32, y1: f32, x2: f32, y2: f32) -> EasingType {
        EasingType::CubicBezier(CubicBezier::new(CubicBezierControls { x1, y1, x2, y2 }))
    }

    pub fn symmetric_cubic_bezier(x1: f32, y1: f32) -> EasingType {
        EasingType::cubic_bezier(x1, y1, 1.0 - x1, 1.0 - y1)
    }

    pub fn apply(&self, x: f32) -> f32 {
        match self {
            EasingType::CONSTANT => 0.0,
            EasingType::LINEAR => x,
            EasingType::CubicBezier(bezier) => bezier.apply(x),
        }
    }

    /// `CODEC`: a simple easing's name, or `{"cubic_bezier": [x1, y1, x2, y2]}`.
    pub fn from_json(json: &JsonElement) -> Result<EasingType, String> {
        if let Some(primitive) = json.as_json_primitive()
            && primitive.is_string()
        {
            return match primitive.get_as_string().as_str() {
                "constant" => Ok(EasingType::CONSTANT),
                "linear" => Ok(EasingType::LINEAR),
                other => Err(format!("the easing {other} (Ease) isn't ported")),
            };
        }
        let controls = json
            .as_json_object()
            .and_then(|object| object.get("cubic_bezier"))
            .and_then(JsonElement::as_json_array)
            .ok_or_else(|| format!("not an easing: {}", json.to_json_string()))?;
        let [x1, y1, x2, y2] = controls else {
            return Err(format!(
                "Input is not a list of 4 elements: {}",
                json.to_json_string()
            ));
        };
        let controls = CubicBezierControls {
            x1: x1.get_as_float(),
            y1: y1.get_as_float(),
            x2: x2.get_as_float(),
            y2: y2.get_as_float(),
        };
        controls.validate()?;
        Ok(EasingType::CubicBezier(CubicBezier::new(controls)))
    }
}

/// `EasingType.CubicBezier`: the curve through (0, 0) and (1, 1) its two controls bend, solved
/// for t by Newton-Raphson, then by bisection.
#[derive(Clone, Copy, Debug)]
pub struct CubicBezier {
    controls: CubicBezierControls,
    x_curve: CubicCurve,
    y_curve: CubicCurve,
}

/// Java compares the controls.
impl PartialEq for CubicBezier {
    fn eq(&self, other: &CubicBezier) -> bool {
        self.controls == other.controls
    }
}

impl CubicBezier {
    const NEWTON_RAPHSON_ITERATIONS: i32 = 4;
    const MAX_STEP: f32 = 0.25;

    pub fn new(controls: CubicBezierControls) -> CubicBezier {
        CubicBezier {
            controls,
            x_curve: CubicBezier::curve_from_controls(controls.x1, controls.x2),
            y_curve: CubicBezier::curve_from_controls(controls.y1, controls.y2),
        }
    }

    fn curve_from_controls(v1: f32, v2: f32) -> CubicCurve {
        CubicCurve {
            a: 3.0 * v1 - 3.0 * v2 + 1.0,
            b: -6.0 * v1 + 3.0 * v2,
            c: 3.0 * v1,
        }
    }

    pub fn apply(&self, x: f32) -> f32 {
        self.y_curve.sample(self.solve_t(x))
    }

    fn solve_t(&self, x: f32) -> f32 {
        let mut t = x;

        for _ in 0..CubicBezier::NEWTON_RAPHSON_ITERATIONS {
            let error = self.x_curve.sample(t) - x;
            if error.abs() < 1.0E-5 {
                return t;
            }

            let gradient = self.x_curve.sample_gradient(t);
            if gradient < 1.0E-5 {
                break;
            }

            t -= Mth::clamp_f32(
                error / gradient,
                -CubicBezier::MAX_STEP,
                CubicBezier::MAX_STEP,
            );
        }

        self.solve_t_bisect(x, t)
    }

    fn solve_t_bisect(&self, x: f32, initial_t: f32) -> f32 {
        let mut t0 = 0.0f32;
        let mut t1 = 1.0f32;

        let mut t = initial_t;
        while t0 < t1 {
            let error = self.x_curve.sample(t) - x;
            if error.abs() < 1.0E-5 {
                return t;
            }

            if error < 0.0 {
                t0 = t;
            } else {
                t1 = t;
            }
            t = (t1 + t0) / 2.0;
        }

        t
    }
}

#[derive(Clone, Copy, Debug)]
struct CubicCurve {
    a: f32,
    b: f32,
    c: f32,
}

impl CubicCurve {
    fn sample(&self, t: f32) -> f32 {
        ((self.a * t + self.b) * t + self.c) * t
    }

    fn sample_gradient(&self, t: f32) -> f32 {
        (3.0 * self.a * t + 2.0 * self.b) * t + self.c
    }
}

/// `EasingType.CubicBezierControls`, a record compared by its floats' `equals`.
#[derive(Clone, Copy, Debug)]
pub struct CubicBezierControls {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

impl PartialEq for CubicBezierControls {
    fn eq(&self, other: &CubicBezierControls) -> bool {
        [self.x1, self.y1, self.x2, self.y2]
            .iter()
            .zip([other.x1, other.y1, other.x2, other.y2])
            .all(|(a, b)| a.to_bits() == b.to_bits() || a.is_nan() && b.is_nan())
    }
}

impl CubicBezierControls {
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    fn validate(&self) -> Result<(), String> {
        if self.x1 < 0.0 || self.x1 > 1.0 {
            Err("x1 must be in range [0; 1]".to_owned())
        } else if !(self.x2 < 0.0) && !(self.x2 > 1.0) {
            Ok(())
        } else {
            Err("x2 must be in range [0; 1]".to_owned())
        }
    }
}
