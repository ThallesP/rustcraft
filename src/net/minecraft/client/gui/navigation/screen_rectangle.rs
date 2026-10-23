//! net/minecraft/client/gui/navigation/ScreenRectangle.java
//!
//! A record of its position and size, compared by value like Java's. The GUI's element states
//! hold their bounds and scissor areas as these, which `GuiRenderState` places elements by.

use crate::java::lang::math::Math;
use crate::net::minecraft::client::gui::navigation::screen_axis::ScreenAxis;
use crate::net::minecraft::client::gui::navigation::screen_direction::ScreenDirection;
use crate::net::minecraft::client::gui::navigation::screen_position::ScreenPosition;
use crate::net::minecraft::util::mth::Mth;
use crate::org::joml::matrix3x2f::Matrix3x2f;
use crate::org::joml::vector2f::Vector2f;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ScreenRectangle {
    pub position: ScreenPosition,
    pub width: i32,
    pub height: i32,
}

impl ScreenRectangle {
    const EMPTY: ScreenRectangle = ScreenRectangle {
        position: ScreenPosition { x: 0, y: 0 },
        width: 0,
        height: 0,
    };

    pub fn new(x: i32, y: i32, width: i32, height: i32) -> ScreenRectangle {
        ScreenRectangle {
            position: ScreenPosition::new(x, y),
            width,
            height,
        }
    }

    pub fn empty() -> ScreenRectangle {
        ScreenRectangle::EMPTY
    }

    pub fn of(
        primary_axis: ScreenAxis,
        primary_index: i32,
        secondary_index: i32,
        primary_length: i32,
        secondary_length: i32,
    ) -> ScreenRectangle {
        match primary_axis {
            ScreenAxis::HORIZONTAL => ScreenRectangle::new(
                primary_index,
                secondary_index,
                primary_length,
                secondary_length,
            ),
            ScreenAxis::VERTICAL => ScreenRectangle::new(
                secondary_index,
                primary_index,
                secondary_length,
                primary_length,
            ),
        }
    }

    pub fn step(&self, direction: ScreenDirection) -> ScreenRectangle {
        ScreenRectangle {
            position: self.position.step(direction),
            width: self.width,
            height: self.height,
        }
    }

    pub fn get_length(&self, axis: ScreenAxis) -> i32 {
        match axis {
            ScreenAxis::HORIZONTAL => self.width,
            ScreenAxis::VERTICAL => self.height,
        }
    }

    pub fn get_bound_in_direction(&self, direction: ScreenDirection) -> i32 {
        let axis = direction.get_axis();
        if direction.is_positive() {
            self.position
                .get_coordinate(axis)
                .wrapping_add(self.get_length(axis))
                .wrapping_sub(1)
        } else {
            self.position.get_coordinate(axis)
        }
    }

    pub fn get_border(&self, direction: ScreenDirection) -> ScreenRectangle {
        let start_first = self.get_bound_in_direction(direction);
        let orthogonal_axis = direction.get_axis().orthogonal();
        let start_second = self.get_bound_in_direction(orthogonal_axis.get_negative());
        let length = self.get_length(orthogonal_axis);
        ScreenRectangle::of(direction.get_axis(), start_first, start_second, 1, length)
            .step(direction)
    }

    pub fn overlaps(&self, other: &ScreenRectangle) -> bool {
        self.overlaps_in_axis(other, ScreenAxis::HORIZONTAL)
            && self.overlaps_in_axis(other, ScreenAxis::VERTICAL)
    }

    pub fn overlaps_in_axis(&self, other: &ScreenRectangle, axis: ScreenAxis) -> bool {
        let this_lower = self.get_bound_in_direction(axis.get_negative());
        let other_lower = other.get_bound_in_direction(axis.get_negative());
        let this_higher = self.get_bound_in_direction(axis.get_positive());
        let other_higher = other.get_bound_in_direction(axis.get_positive());
        this_lower.max(other_lower) <= this_higher.min(other_higher)
    }

    pub fn get_center_in_axis(&self, axis: ScreenAxis) -> i32 {
        self.get_bound_in_direction(axis.get_positive())
            .wrapping_add(self.get_bound_in_direction(axis.get_negative()))
            / 2
    }

    pub fn intersection(&self, other: &ScreenRectangle) -> Option<ScreenRectangle> {
        let left = self.left().max(other.left());
        let top = self.top().max(other.top());
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());
        (left < right && top < bottom).then(|| {
            ScreenRectangle::new(
                left,
                top,
                right.wrapping_sub(left),
                bottom.wrapping_sub(top),
            )
        })
    }

    pub fn intersects(&self, other: &ScreenRectangle) -> bool {
        self.left() < other.right()
            && self.right() > other.left()
            && self.top() < other.bottom()
            && self.bottom() > other.top()
    }

    pub fn encompasses(&self, other: &ScreenRectangle) -> bool {
        other.left() >= self.left()
            && other.top() >= self.top()
            && other.right() <= self.right()
            && other.bottom() <= self.bottom()
    }

    pub fn top(&self) -> i32 {
        self.position.y
    }

    pub fn bottom(&self) -> i32 {
        self.position.y.wrapping_add(self.height)
    }

    pub fn left(&self) -> i32 {
        self.position.x
    }

    pub fn right(&self) -> i32 {
        self.position.x.wrapping_add(self.width)
    }

    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        x >= self.left() && x < self.right() && y >= self.top() && y < self.bottom()
    }

    pub fn transform_axis_aligned(&self, matrix: &Matrix3x2f) -> ScreenRectangle {
        let mut top_left = Vector2f::new();
        matrix.transform_position_xy_into(self.left() as f32, self.top() as f32, &mut top_left);
        let mut bottom_right = Vector2f::new();
        matrix.transform_position_xy_into(
            self.right() as f32,
            self.bottom() as f32,
            &mut bottom_right,
        );
        ScreenRectangle::new(
            Mth::floor_f32(top_left.x),
            Mth::floor_f32(top_left.y),
            Mth::floor_f32(bottom_right.x - top_left.x),
            Mth::floor_f32(bottom_right.y - top_left.y),
        )
    }

    pub fn transform_max_bounds(&self, matrix: &Matrix3x2f) -> ScreenRectangle {
        let corner = |x: i32, y: i32| {
            let mut corner = Vector2f::new();
            matrix.transform_position_xy_into(x as f32, y as f32, &mut corner);
            corner
        };
        let top_left = corner(self.left(), self.top());
        let top_right = corner(self.right(), self.top());
        let bottom_left = corner(self.left(), self.bottom());
        let bottom_right = corner(self.right(), self.bottom());
        let min_x = Math::min_f32(
            Math::min_f32(top_left.x, bottom_left.x),
            Math::min_f32(top_right.x, bottom_right.x),
        );
        let max_x = Math::max_f32(
            Math::max_f32(top_left.x, bottom_left.x),
            Math::max_f32(top_right.x, bottom_right.x),
        );
        let min_y = Math::min_f32(
            Math::min_f32(top_left.y, bottom_left.y),
            Math::min_f32(top_right.y, bottom_right.y),
        );
        let max_y = Math::max_f32(
            Math::max_f32(top_left.y, bottom_left.y),
            Math::max_f32(top_right.y, bottom_right.y),
        );
        ScreenRectangle::new(
            Mth::floor_f32(min_x),
            Mth::floor_f32(min_y),
            Mth::ceil_f32(max_x - min_x),
            Mth::ceil_f32(max_y - min_y),
        )
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::golden::{Decode, Encode, Golden};
    use serde_json::{Value, json};

    impl Decode for ScreenRectangle {
        fn decode(value: &Value) -> ScreenRectangle {
            let [x, y, width, height] = <[i32; 4]>::decode(value);
            ScreenRectangle::new(x, y, width, height)
        }
    }

    impl Encode for ScreenRectangle {
        fn encode(&self) -> Value {
            json!([self.left(), self.top(), self.width, self.height])
        }
    }

    /// A pose as `GuiSuites` writes one: its six floats' bits.
    pub(crate) fn pose(value: &Value) -> Matrix3x2f {
        let [m00, m01, m10, m11, m20, m21] = <[f32; 6]>::decode(value);
        Matrix3x2f {
            m00,
            m01,
            m10,
            m11,
            m20,
            m21,
        }
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.client.gui.navigation.ScreenRectangle");
        golden.check("intersection", |a: ScreenRectangle, b: ScreenRectangle| {
            a.intersection(&b)
        });
        golden.check("intersects", |a: ScreenRectangle, b: ScreenRectangle| {
            a.intersects(&b)
        });
        golden.check("encompasses", |a: ScreenRectangle, b: ScreenRectangle| {
            a.encompasses(&b)
        });
        golden.check("overlaps", |a: ScreenRectangle, b: ScreenRectangle| {
            a.overlaps(&b)
        });
        golden.check("containsPoint", |a: ScreenRectangle, x: i32, y: i32| {
            a.contains_point(x, y)
        });
        golden.check("getBorder", |a: ScreenRectangle, direction: String| {
            let direction = ScreenDirection::VALUES
                .into_iter()
                .find(|value| format!("{value:?}") == direction)
                .expect("a direction");
            a.get_border(direction)
        });
        golden.check("transformMaxBounds", |a: ScreenRectangle, matrix: Value| {
            a.transform_max_bounds(&pose(&matrix))
        });
        golden.check(
            "transformAxisAligned",
            |a: ScreenRectangle, matrix: Value| a.transform_axis_aligned(&pose(&matrix)),
        );
        golden.finish();
    }
}
