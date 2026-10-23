//! net/minecraft/client/model/geom/builders/CubeListBuilder.java
//!
//! The overloads that take a cube's id (a comment only the debug tools read) aren't ported; the
//! others are told apart by what they add: `add_box_deformed` the growth, `add_box_faces` the
//! visible faces, `add_box_mirrored` the mirroring, `add_box_scaled` the growth and the texture's
//! scale.

use crate::net::minecraft::client::model::geom::builders::cube_definition::CubeDefinition;
use crate::net::minecraft::client::model::geom::builders::cube_deformation::CubeDeformation;
use crate::net::minecraft::core::direction::Direction;

/// `EnumSet.allOf(Direction.class)`.
const ALL_VISIBLE: &[Direction] = &Direction::VALUES;

/// A part's cubes as they're added, each with the texture offset and mirroring set before it.
#[derive(Clone, Debug, Default)]
pub struct CubeListBuilder {
    cubes: Vec<CubeDefinition>,
    x_tex_offs: i32,
    y_tex_offs: i32,
    mirror: bool,
}

impl CubeListBuilder {
    pub fn create() -> CubeListBuilder {
        CubeListBuilder::default()
    }

    pub fn tex_offs(mut self, x_tex_offs: i32, y_tex_offs: i32) -> CubeListBuilder {
        self.x_tex_offs = x_tex_offs;
        self.y_tex_offs = y_tex_offs;
        self
    }

    /// `mirror()`.
    pub fn mirror(self) -> CubeListBuilder {
        self.set_mirror(true)
    }

    /// `mirror(boolean)`.
    pub fn set_mirror(mut self, mirror: bool) -> CubeListBuilder {
        self.mirror = mirror;
        self
    }

    #[allow(clippy::too_many_arguments)]
    fn push(
        mut self,
        x0: f32,
        y0: f32,
        z0: f32,
        w: f32,
        h: f32,
        d: f32,
        g: CubeDeformation,
        mirror: bool,
        x_tex_scale: f32,
        y_tex_scale: f32,
        visible_sides: &[Direction],
    ) -> CubeListBuilder {
        self.cubes.push(CubeDefinition::new(
            self.x_tex_offs as f32,
            self.y_tex_offs as f32,
            x0,
            y0,
            z0,
            w,
            h,
            d,
            g,
            mirror,
            x_tex_scale,
            y_tex_scale,
            visible_sides,
        ));
        self
    }

    pub fn add_box(self, x0: f32, y0: f32, z0: f32, w: f32, h: f32, d: f32) -> CubeListBuilder {
        let mirror = self.mirror;
        self.push(
            x0,
            y0,
            z0,
            w,
            h,
            d,
            CubeDeformation::NONE,
            mirror,
            1.0,
            1.0,
            ALL_VISIBLE,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_box_faces(
        self,
        x0: f32,
        y0: f32,
        z0: f32,
        w: f32,
        h: f32,
        d: f32,
        visible_sides: &[Direction],
    ) -> CubeListBuilder {
        let mirror = self.mirror;
        self.push(
            x0,
            y0,
            z0,
            w,
            h,
            d,
            CubeDeformation::NONE,
            mirror,
            1.0,
            1.0,
            visible_sides,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_box_mirrored(
        self,
        x0: f32,
        y0: f32,
        z0: f32,
        w: f32,
        h: f32,
        d: f32,
        mirror: bool,
    ) -> CubeListBuilder {
        self.push(
            x0,
            y0,
            z0,
            w,
            h,
            d,
            CubeDeformation::NONE,
            mirror,
            1.0,
            1.0,
            ALL_VISIBLE,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_box_scaled(
        self,
        x0: f32,
        y0: f32,
        z0: f32,
        w: f32,
        h: f32,
        d: f32,
        g: CubeDeformation,
        x_tex_scale: f32,
        y_tex_scale: f32,
    ) -> CubeListBuilder {
        let mirror = self.mirror;
        self.push(
            x0,
            y0,
            z0,
            w,
            h,
            d,
            g,
            mirror,
            x_tex_scale,
            y_tex_scale,
            ALL_VISIBLE,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_box_deformed(
        self,
        x0: f32,
        y0: f32,
        z0: f32,
        w: f32,
        h: f32,
        d: f32,
        g: CubeDeformation,
    ) -> CubeListBuilder {
        let mirror = self.mirror;
        self.push(x0, y0, z0, w, h, d, g, mirror, 1.0, 1.0, ALL_VISIBLE)
    }

    pub fn get_cubes(&self) -> Vec<CubeDefinition> {
        self.cubes.clone()
    }
}
