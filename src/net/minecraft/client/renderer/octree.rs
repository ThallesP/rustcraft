//! net/minecraft/client/renderer/Octree.java
//!
//! The visible sections in a tree of boxes around the camera, each branch's children ordered
//! nearest the camera first, so a walk visits the sections in front to back order and culls whole
//! boxes against the frustum. A leaf is a section's index in the view area, whose bounding box the
//! visitor is given.

use crate::net::minecraft::client::renderer::culling::frustum::Frustum;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::section_pos::SectionPos;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::level::levelgen::structure::bounding_box::BoundingBox;
use crate::net::minecraft::world::phys::aabb::AABB;
use crate::org::joml::frustum_intersection::FrustumIntersection;

pub struct Octree {
    root: Branch,
    camera_section_center: BlockPos,
}

/// `Octree.AxisSorting`: which axis's side counts most in a child's index.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AxisSorting {
    XYZ,
    XZY,
    YXZ,
    YZX,
    ZXY,
    ZYX,
}

impl AxisSorting {
    /// `(xShift, yShift, zShift)`.
    fn shifts(self) -> (usize, usize, usize) {
        match self {
            AxisSorting::XYZ => (4, 2, 1),
            AxisSorting::XZY => (4, 1, 2),
            AxisSorting::YXZ => (2, 4, 1),
            AxisSorting::YZX => (1, 4, 2),
            AxisSorting::ZXY => (2, 1, 4),
            AxisSorting::ZYX => (1, 2, 4),
        }
    }

    fn get_axis_sorting(abs_x_diff: i32, abs_y_diff: i32, abs_z_diff: i32) -> AxisSorting {
        if abs_x_diff > abs_y_diff && abs_x_diff > abs_z_diff {
            if abs_y_diff > abs_z_diff {
                AxisSorting::XYZ
            } else {
                AxisSorting::XZY
            }
        } else if abs_y_diff > abs_x_diff && abs_y_diff > abs_z_diff {
            if abs_x_diff > abs_z_diff {
                AxisSorting::YXZ
            } else {
                AxisSorting::YZX
            }
        } else if abs_x_diff > abs_y_diff {
            AxisSorting::ZXY
        } else {
            AxisSorting::ZYX
        }
    }
}

enum Node {
    Branch(Box<Branch>),
    /// A section's index in the view area, and its bounding box.
    Leaf(usize, AABB),
}

struct Branch {
    nodes: [Option<Node>; 8],
    bounding_box: BoundingBox,
    bb_center_x: i32,
    bb_center_y: i32,
    bb_center_z: i32,
    sorting: AxisSorting,
    camera_x_diff_negative: bool,
    camera_y_diff_negative: bool,
    camera_z_diff_negative: bool,
}

impl Octree {
    pub fn new(
        camera_section: SectionPos,
        render_distance: i32,
        sections_per_chunk: i32,
        min_block_y: i32,
    ) -> Octree {
        let visible_area_diameter_in_sections = render_distance * 2 + 1;
        let bounding_box_size_in_sections =
            Mth::smallest_encompassing_power_of_two(visible_area_diameter_in_sections);
        let distance_to_bb_edge_in_blocks = render_distance * 16;
        let camera_section_origin = BlockPos::new(
            camera_section.min_block_x(),
            camera_section.min_block_y(),
            camera_section.min_block_z(),
        );
        let camera_section_center = camera_section_origin.offset_xyz(8, 8, 8);
        let min_x = camera_section_origin.get_x() - distance_to_bb_edge_in_blocks;
        let max_x = min_x + bounding_box_size_in_sections * 16 - 1;
        let min_y = if bounding_box_size_in_sections >= sections_per_chunk {
            min_block_y
        } else {
            camera_section_origin.get_y() - distance_to_bb_edge_in_blocks
        };
        let max_y = min_y + bounding_box_size_in_sections * 16 - 1;
        let min_z = camera_section_origin.get_z() - distance_to_bb_edge_in_blocks;
        let max_z = min_z + bounding_box_size_in_sections * 16 - 1;
        Octree {
            root: Branch::new(
                BoundingBox::new(min_x, min_y, min_z, max_x, max_y, max_z),
                &camera_section_center,
            ),
            camera_section_center,
        }
    }

    /// Adds the section at `section_node`, whose index in the view area is `index` and bounding
    /// box `bb`. False if one was there already.
    pub fn add(&mut self, index: usize, section_node: i64, bb: AABB) -> bool {
        let center = self.camera_section_center;
        self.root.add(index, section_node, bb, &center)
    }

    /// Visits each node in the frustum, a branch before its children: the visitor gets the
    /// section's index for a leaf (none for a branch) and whether it's within `close_distance`
    /// of the camera's section.
    pub fn visit_nodes(
        &self,
        visitor: &mut dyn FnMut(Option<usize>, bool, i32, bool),
        frustum: &Frustum,
        close_distance: i32,
    ) {
        self.root
            .visit_nodes(self, visitor, false, frustum, 0, close_distance, true);
    }

    #[allow(clippy::too_many_arguments)]
    fn is_close(
        &self,
        min_x: f64,
        min_y: f64,
        min_z: f64,
        max_x: f64,
        max_y: f64,
        max_z: f64,
        close_distance: i32,
    ) -> bool {
        let camera_x = f64::from(self.camera_section_center.get_x());
        let camera_y = f64::from(self.camera_section_center.get_y());
        let camera_z = f64::from(self.camera_section_center.get_z());
        let close_distance = f64::from(close_distance);
        camera_x > min_x - close_distance
            && camera_x < max_x + close_distance
            && camera_y > min_y - close_distance
            && camera_y < max_y + close_distance
            && camera_z > min_z - close_distance
            && camera_z < max_z + close_distance
    }
}

impl Branch {
    fn new(bounding_box: BoundingBox, camera_section_center: &BlockPos) -> Branch {
        let bb_center_x = bounding_box.min_x() + bounding_box.get_x_span() / 2;
        let bb_center_y = bounding_box.min_y() + bounding_box.get_y_span() / 2;
        let bb_center_z = bounding_box.min_z() + bounding_box.get_z_span() / 2;
        let camera_x_diff = camera_section_center.get_x() - bb_center_x;
        let camera_y_diff = camera_section_center.get_y() - bb_center_y;
        let camera_z_diff = camera_section_center.get_z() - bb_center_z;
        Branch {
            nodes: Default::default(),
            bounding_box,
            bb_center_x,
            bb_center_y,
            bb_center_z,
            sorting: AxisSorting::get_axis_sorting(
                camera_x_diff.abs(),
                camera_y_diff.abs(),
                camera_z_diff.abs(),
            ),
            camera_x_diff_negative: camera_x_diff < 0,
            camera_y_diff_negative: camera_y_diff < 0,
            camera_z_diff_negative: camera_z_diff < 0,
        }
    }

    fn add(
        &mut self,
        index: usize,
        section_node: i64,
        bb: AABB,
        camera_section_center: &BlockPos,
    ) -> bool {
        let section_x_diff_negative =
            SectionPos::section_to_block_coord(SectionPos::x_of(section_node)) - self.bb_center_x
                < 0;
        let section_y_diff_negative =
            SectionPos::section_to_block_coord(SectionPos::y_of(section_node)) - self.bb_center_y
                < 0;
        let section_z_diff_negative =
            SectionPos::section_to_block_coord(SectionPos::z_of(section_node)) - self.bb_center_z
                < 0;
        let node_index = Branch::get_node_index(
            self.sorting,
            section_x_diff_negative != self.camera_x_diff_negative,
            section_y_diff_negative != self.camera_y_diff_negative,
            section_z_diff_negative != self.camera_z_diff_negative,
        );
        if self.are_children_leaves() {
            let already_existed = self.nodes[node_index].is_some();
            self.nodes[node_index] = Some(Node::Leaf(index, bb));
            return !already_existed;
        }

        if let Some(Node::Branch(branch)) = &mut self.nodes[node_index] {
            return branch.add(index, section_node, bb, camera_section_center);
        }

        let child_bounding_box = self.create_child_bounding_box(
            section_x_diff_negative,
            section_y_diff_negative,
            section_z_diff_negative,
        );
        let mut branch = Box::new(Branch::new(child_bounding_box, camera_section_center));
        let added = branch.add(index, section_node, bb, camera_section_center);
        self.nodes[node_index] = Some(Node::Branch(branch));
        added
    }

    fn get_node_index(
        sorting: AxisSorting,
        x_diffs_opposite_sides: bool,
        y_diffs_opposite_sides: bool,
        z_diffs_opposite_sides: bool,
    ) -> usize {
        let (x_shift, y_shift, z_shift) = sorting.shifts();
        let mut index = 0;
        if x_diffs_opposite_sides {
            index += x_shift;
        }
        if y_diffs_opposite_sides {
            index += y_shift;
        }
        if z_diffs_opposite_sides {
            index += z_shift;
        }
        index
    }

    fn are_children_leaves(&self) -> bool {
        self.bounding_box.get_x_span() == 32
    }

    fn create_child_bounding_box(
        &self,
        section_x_diff_negative: bool,
        section_y_diff_negative: bool,
        section_z_diff_negative: bool,
    ) -> BoundingBox {
        let (min_x, max_x) = if section_x_diff_negative {
            (self.bounding_box.min_x(), self.bb_center_x - 1)
        } else {
            (self.bb_center_x, self.bounding_box.max_x())
        };
        let (min_y, max_y) = if section_y_diff_negative {
            (self.bounding_box.min_y(), self.bb_center_y - 1)
        } else {
            (self.bb_center_y, self.bounding_box.max_y())
        };
        let (min_z, max_z) = if section_z_diff_negative {
            (self.bounding_box.min_z(), self.bb_center_z - 1)
        } else {
            (self.bb_center_z, self.bounding_box.max_z())
        };
        BoundingBox::new(min_x, min_y, min_z, max_x, max_y, max_z)
    }

    #[allow(clippy::too_many_arguments)]
    fn visit_nodes(
        &self,
        octree: &Octree,
        visitor: &mut dyn FnMut(Option<usize>, bool, i32, bool),
        mut skip_frustum_check: bool,
        frustum: &Frustum,
        depth: i32,
        close_distance: i32,
        is_close: bool,
    ) {
        let mut is_visible = skip_frustum_check;
        if !skip_frustum_check {
            let check_result = frustum.cube_in_frustum(&self.bounding_box);
            skip_frustum_check = check_result == FrustumIntersection::INSIDE;
            is_visible = check_result == FrustumIntersection::INSIDE
                || check_result == FrustumIntersection::INTERSECT;
        }

        if is_visible {
            let bb = &self.bounding_box;
            let is_close = is_close
                && octree.is_close(
                    f64::from(bb.min_x()),
                    f64::from(bb.min_y()),
                    f64::from(bb.min_z()),
                    f64::from(bb.max_x()),
                    f64::from(bb.max_y()),
                    f64::from(bb.max_z()),
                    close_distance,
                );
            visitor(None, skip_frustum_check, depth, is_close);
            for node in self.nodes.iter().flatten() {
                match node {
                    Node::Branch(branch) => branch.visit_nodes(
                        octree,
                        visitor,
                        skip_frustum_check,
                        frustum,
                        depth + 1,
                        close_distance,
                        is_close,
                    ),
                    Node::Leaf(index, bb) => {
                        if skip_frustum_check || frustum.is_visible(bb) {
                            let is_close = is_close
                                && octree.is_close(
                                    bb.min_x,
                                    bb.min_y,
                                    bb.min_z,
                                    bb.max_x,
                                    bb.max_y,
                                    bb.max_z,
                                    close_distance,
                                );
                            visitor(Some(*index), skip_frustum_check, depth + 1, is_close);
                        }
                    }
                }
            }
        }
    }
}
