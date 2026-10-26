//! net/minecraft/client/renderer/SectionOcclusionGraph.java
//!
//! Which sections the camera can see: a breadth-first walk from the camera's section through the
//! faces each section's mesh sees between, kept in an octree for the frustum walk. Java runs the
//! full walk on a background thread and the frame after uses it; here it runs when it's asked
//! for, which only changes how soon its result shows. Sections are their indices in the view
//! area. Not ported yet: a captured frustum, the debug renderer's access to the nodes, and the
//! chunk cache's sets of added and removed chunks and empty sections, which the level renderer
//! works out from the level each frame instead (`update`'s arguments).

use std::collections::{HashMap, HashSet, VecDeque};

use crate::it::unimi::dsi::fastutil::hash_common::BuildMixHasher;

use crate::net::minecraft::client::renderer::culling::frustum::Frustum;
use crate::net::minecraft::client::renderer::state::level::camera_render_state::CameraRenderState;
use crate::net::minecraft::client::renderer::view_area::ViewArea;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::{Axis, Direction};
use crate::net::minecraft::core::section_pos::SectionPos;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::phys::aabb::AABB;
use crate::net::minecraft::world::phys::vec3::Vec3;
use crate::org::joml::vector3d::Vector3d;

/// A set of section or chunk nodes, hashed with fastutil's `mix` like Java's `LongOpenHashSet`:
/// the graph only asks it what it holds.
pub type LongSet = HashSet<i64, BuildMixHasher>;

pub struct SectionOcclusionGraph {
    needs_full_update: bool,
    current_graph: Option<GraphState>,
    needs_frustum_update: bool,
    empty_sections: LongSet,
    loaded_chunks: LongSet,
    prev_cam_x: f64,
    prev_cam_y: f64,
    prev_cam_z: f64,
    prev_fov: i32,
    last_smart_cull: bool,
}

struct GraphState {
    storage: GraphStorage,
    sections_to_propagate_from: VecDeque<usize>,
}

struct GraphStorage {
    /// `SectionToNodeMap`: each section's node, by its index.
    section_to_node_map: Vec<Option<Node>>,
    section_tree: crate::net::minecraft::client::renderer::octree::Octree,
    sections_waiting_for_chunk_loads: HashMap<i64, Vec<i64>>,
}

/// `SectionOcclusionGraph.Node`: a reached section, the faces it was reached through and the
/// directions the walk went to get there.
#[derive(Clone, Copy, Debug)]
pub struct Node {
    section: usize,
    source_directions: u8,
    directions: u8,
    pub step: i32,
}

impl Node {
    fn new(section: usize, source_direction: Option<Direction>, step: i32) -> Node {
        let mut node = Node {
            section,
            source_directions: 0,
            directions: 0,
            step,
        };
        if let Some(source_direction) = source_direction {
            node.add_source_direction(source_direction);
        }
        node
    }

    fn set_directions(&mut self, old_directions: u8, direction: Direction) {
        self.directions |= old_directions | 1 << direction as u8;
    }

    fn has_direction(&self, direction: Direction) -> bool {
        self.directions & 1 << direction as u8 != 0
    }

    fn add_source_direction(&mut self, direction: Direction) {
        self.source_directions |= 1 << direction as u8;
    }

    pub fn has_source_direction(&self, direction_ordinal: usize) -> bool {
        self.source_directions & 1 << direction_ordinal != 0
    }

    fn has_source_directions(&self) -> bool {
        self.source_directions != 0
    }
}

impl GraphStorage {
    fn new(view_area: &ViewArea) -> GraphStorage {
        GraphStorage {
            section_to_node_map: vec![None; view_area.size()],
            section_tree: crate::net::minecraft::client::renderer::octree::Octree::new(
                view_area.get_camera_section_pos(),
                view_area.get_view_distance(),
                view_area.section_count(),
                view_area.min_y(),
            ),
            sections_waiting_for_chunk_loads: HashMap::new(),
        }
    }
}

impl Default for SectionOcclusionGraph {
    fn default() -> SectionOcclusionGraph {
        SectionOcclusionGraph::new()
    }
}

impl SectionOcclusionGraph {
    const MINIMUM_ADVANCED_CULLING_SECTION_DISTANCE: i32 = 60 >> 4;

    pub fn new() -> SectionOcclusionGraph {
        SectionOcclusionGraph {
            needs_full_update: true,
            current_graph: None,
            needs_frustum_update: false,
            empty_sections: LongSet::default(),
            loaded_chunks: LongSet::default(),
            prev_cam_x: f64::from_bits(1),
            prev_cam_y: f64::from_bits(1),
            prev_cam_z: f64::from_bits(1),
            prev_fov: i32::MAX,
            last_smart_cull: true,
        }
    }

    fn ceiled_section_diagonal() -> f64 {
        (3.0f64.sqrt() * 16.0).ceil()
    }

    pub fn wait_and_reset(&mut self, view_area: Option<&ViewArea>) {
        match view_area {
            Some(view_area) => {
                self.current_graph = Some(GraphState {
                    storage: GraphStorage::new(view_area),
                    sections_to_propagate_from: VecDeque::new(),
                });
                self.invalidate();
            }
            None => {
                self.current_graph = None;
                self.empty_sections.clear();
                self.loaded_chunks.clear();
            }
        }
    }

    pub fn invalidate(&mut self) {
        self.needs_full_update = true;
    }

    fn invalidate_if_needed(&mut self, camera: &CameraRenderState, fov: i32) {
        let camera_pos = camera.pos;
        let cam_x = (camera_pos.x / 8.0).floor();
        let cam_y = (camera_pos.y / 8.0).floor();
        let cam_z = (camera_pos.z / 8.0).floor();
        if cam_x != self.prev_cam_x
            || cam_y != self.prev_cam_y
            || cam_z != self.prev_cam_z
            || self.prev_fov != fov
            || self.last_smart_cull != camera.smart_cull
        {
            self.invalidate();
        }

        self.prev_cam_x = cam_x;
        self.prev_cam_y = cam_y;
        self.prev_cam_z = cam_z;
        self.prev_fov = fov;
        self.last_smart_cull = camera.smart_cull;
    }

    /// The sections in the frustum, nearest first, and those within 32 blocks of the camera's
    /// section.
    pub fn add_sections_in_frustum(
        &self,
        frustum: &Frustum,
        visible_sections: &mut Vec<usize>,
        nearby_visible_sections: &mut Vec<usize>,
    ) {
        let Some(graph) = &self.current_graph else {
            return;
        };
        let offset_frustum = SectionOcclusionGraph::offset_frustum(frustum);
        graph.storage.section_tree.visit_nodes(
            &mut |section, _fully_visible, _depth, is_close| {
                if let Some(section) = section {
                    visible_sections.push(section);
                    if is_close {
                        nearby_visible_sections.push(section);
                    }
                }
            },
            &offset_frustum,
            32,
        );
    }

    pub fn consume_frustum_update(&mut self) -> bool {
        std::mem::take(&mut self.needs_frustum_update)
    }

    /// A section's mesh changed: the walk goes on from it.
    pub fn schedule_propagation_from(&mut self, section: usize) {
        if let Some(graph) = &mut self.current_graph {
            graph.sections_to_propagate_from.push_back(section);
        }
    }

    /// The frame's update: the level's loaded chunks and empty sections, a full walk when the
    /// camera moved or the graph was invalidated, then the walk on from the sections whose
    /// meshes changed or whose chunks came.
    pub fn update(
        &mut self,
        camera: &CameraRenderState,
        fov: i32,
        view_area: &mut ViewArea,
        loaded_chunks: LongSet,
        empty_sections: LongSet,
    ) {
        let loaded_expected_chunks: Vec<i64> = self
            .current_graph
            .iter()
            .flat_map(|graph| graph.storage.sections_waiting_for_chunk_loads.keys())
            .filter(|chunk| loaded_chunks.contains(chunk))
            .copied()
            .collect();
        self.loaded_chunks = loaded_chunks;
        self.empty_sections = empty_sections;
        if !camera.is_frustum_captured {
            self.invalidate_if_needed(camera, fov);
            if self.needs_full_update {
                self.run_full_update(camera, view_area);
            }

            self.run_partial_update(camera, view_area, &loaded_expected_chunks);
        }
    }

    fn run_full_update(&mut self, camera: &CameraRenderState, view_area: &mut ViewArea) {
        self.needs_full_update = false;
        let mut new_state = GraphState {
            storage: GraphStorage::new(view_area),
            sections_to_propagate_from: VecDeque::new(),
        };
        let mut queue = VecDeque::new();
        self.initialize_queue_for_full_update(view_area, &camera.block_pos, &mut queue);
        for node in &queue {
            new_state.storage.section_to_node_map[node.section] = Some(*node);
        }
        let queue = queue.into_iter().map(|node| node.section).collect();
        SectionOcclusionGraph::run_updates(
            view_area,
            &mut new_state.storage,
            &camera.pos,
            queue,
            camera.smart_cull,
            &mut |_| {},
            &self.empty_sections,
            &self.loaded_chunks,
        );
        self.current_graph = Some(new_state);
        self.needs_frustum_update = true;
    }

    fn run_partial_update(
        &mut self,
        camera: &CameraRenderState,
        view_area: &mut ViewArea,
        loaded_expected_chunks: &[i64],
    ) {
        let Some(state) = &mut self.current_graph else {
            return;
        };
        for chunk_node in loaded_expected_chunks {
            if let Some(waiting_sections) = state
                .storage
                .sections_waiting_for_chunk_loads
                .remove(chunk_node)
            {
                for section_node in waiting_sections {
                    if let Some(section) = view_area.get_render_section(section_node) {
                        state.sections_to_propagate_from.push_back(section);
                    }
                }
            }
        }

        if !state.sections_to_propagate_from.is_empty() {
            let mut queue = VecDeque::new();
            while let Some(render_section) = state.sections_to_propagate_from.pop_front() {
                if state.storage.section_to_node_map[render_section].is_some() {
                    queue.push_back(render_section);
                }
            }

            let offset_frustum = SectionOcclusionGraph::offset_frustum(&camera.cull_frustum);
            let mut needs_frustum_update = false;
            SectionOcclusionGraph::run_updates(
                view_area,
                &mut state.storage,
                &camera.pos,
                queue,
                camera.smart_cull,
                &mut |bounding_box| {
                    if offset_frustum.is_visible(bounding_box) {
                        needs_frustum_update = true;
                    }
                },
                &self.empty_sections,
                &self.loaded_chunks,
            );
            self.needs_frustum_update |= needs_frustum_update;
        }
    }

    fn initialize_queue_for_full_update(
        &self,
        view_area: &ViewArea,
        camera_position: &BlockPos,
        queue: &mut VecDeque<Node>,
    ) {
        let camera_section_node = SectionPos::as_long_xyz(
            SectionPos::block_to_section_coord(camera_position.get_x()),
            SectionPos::block_to_section_coord(camera_position.get_y()),
            SectionPos::block_to_section_coord(camera_position.get_z()),
        );
        let camera_section_y = SectionPos::y_of(camera_section_node);
        match view_area.get_render_section(camera_section_node) {
            None => {
                let is_below_the_world = camera_section_y < view_area.min_section_y();
                let section_y = if is_below_the_world {
                    view_area.min_section_y()
                } else {
                    view_area.max_section_y()
                };
                let view_distance = view_area.get_view_distance();
                let mut to_add = Vec::new();
                let camera_section_x = SectionPos::x_of(camera_section_node);
                let camera_section_z = SectionPos::z_of(camera_section_node);
                for section_x in -view_distance..=view_distance {
                    for section_z in -view_distance..=view_distance {
                        let Some(render_section_at) =
                            view_area.get_render_section(SectionPos::as_long_xyz(
                                section_x + camera_section_x,
                                section_y,
                                section_z + camera_section_z,
                            ))
                        else {
                            continue;
                        };
                        let section_node = view_area.section(render_section_at).section_node();
                        if !self.is_in_view_distance(view_area, camera_section_node, section_node) {
                            continue;
                        }
                        let source_direction = if is_below_the_world {
                            Direction::UP
                        } else {
                            Direction::DOWN
                        };
                        let mut node = Node::new(render_section_at, Some(source_direction), 0);
                        node.set_directions(node.directions, source_direction);
                        if section_x > 0 {
                            node.set_directions(node.directions, Direction::EAST);
                        } else if section_x < 0 {
                            node.set_directions(node.directions, Direction::WEST);
                        }
                        if section_z > 0 {
                            node.set_directions(node.directions, Direction::SOUTH);
                        } else if section_z < 0 {
                            node.set_directions(node.directions, Direction::NORTH);
                        }
                        to_add.push(node);
                    }
                }
                // Comparator.comparingDouble, a stable sort.
                let distance = |node: &Node| {
                    let center =
                        SectionPos::of_long(view_area.section(node.section).section_node());
                    camera_position.dist_sqr(&BlockPos::new(
                        center.min_block_x() + 8,
                        center.min_block_y() + 8,
                        center.min_block_z() + 8,
                    ))
                };
                to_add.sort_by(|a, b| distance(a).total_cmp(&distance(b)));
                queue.extend(to_add);
            }
            Some(camera_section) => queue.push_back(Node::new(camera_section, None, 0)),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn run_updates(
        view_area: &mut ViewArea,
        storage: &mut GraphStorage,
        camera_pos: &Vec3,
        mut queue: VecDeque<usize>,
        smart_cull: bool,
        on_section_added: &mut dyn FnMut(&AABB),
        empty_sections: &LongSet,
        loaded_chunks: &LongSet,
    ) {
        let camera_section_pos = SectionPos::of_block_pos(&BlockPos::containing_xyz(
            camera_pos.x,
            camera_pos.y,
            camera_pos.z,
        ));
        let camera_section_node = camera_section_pos.as_long();
        let camera_section_center = BlockPos::new(
            camera_section_pos.min_block_x() + 8,
            camera_section_pos.min_block_y() + 8,
            camera_section_pos.min_block_z() + 8,
        );

        while let Some(node_section) = queue.pop_front() {
            let node =
                storage.section_to_node_map[node_section].expect("a queued section has a node");
            let section_node = view_area.section(node.section).section_node();
            let chunk_node = SectionOcclusionGraph::chunk_node(section_node);
            if !loaded_chunks.contains(&chunk_node) {
                storage
                    .sections_waiting_for_chunk_loads
                    .entry(chunk_node)
                    .or_default()
                    .push(section_node);
                continue;
            }

            if !empty_sections.contains(&section_node) {
                let bounding_box = *view_area.section(node.section).get_bounding_box();
                if storage
                    .section_tree
                    .add(node.section, section_node, bounding_box)
                {
                    on_section_added(&bounding_box);
                }
            } else {
                // A section of only air sees through all its faces, like a compiled empty one.
                let section = view_area.sections.get_mut(node.section);
                if section.set_empty_if_uncompiled() {
                    section.update_upload_time();
                }
            }
            let current_section = view_area.section(node.section);

            let distant_from_camera = (SectionPos::x_of(section_node) - camera_section_pos.x())
                .abs()
                > SectionOcclusionGraph::MINIMUM_ADVANCED_CULLING_SECTION_DISTANCE
                || (SectionPos::y_of(section_node) - camera_section_pos.y()).abs()
                    > SectionOcclusionGraph::MINIMUM_ADVANCED_CULLING_SECTION_DISTANCE
                || (SectionPos::z_of(section_node) - camera_section_pos.z()).abs()
                    > SectionOcclusionGraph::MINIMUM_ADVANCED_CULLING_SECTION_DISTANCE;

            for direction in Direction::VALUES {
                let Some(render_section_at) = SectionOcclusionGraph::get_relative_from(
                    view_area,
                    camera_section_node,
                    current_section,
                    direction,
                ) else {
                    continue;
                };
                if smart_cull && node.has_direction(direction.get_opposite()) {
                    continue;
                }

                if smart_cull && node.has_source_directions() {
                    let section_mesh = current_section.get_section_mesh();
                    let visible = (0..Direction::VALUES.len()).any(|i| {
                        node.has_source_direction(i)
                            && section_mesh.faces_can_see_eachother(
                                Direction::VALUES[i].get_opposite(),
                                direction,
                            )
                    });
                    if !visible {
                        continue;
                    }
                }

                if smart_cull && distant_from_camera {
                    let origin_x =
                        SectionPos::section_to_block_coord(SectionPos::x_of(section_node));
                    let origin_y =
                        SectionPos::section_to_block_coord(SectionPos::y_of(section_node));
                    let origin_z =
                        SectionPos::section_to_block_coord(SectionPos::z_of(section_node));
                    let axis = direction.get_axis();
                    let max_x = if axis == Axis::X {
                        camera_section_center.get_x() > origin_x
                    } else {
                        camera_section_center.get_x() < origin_x
                    };
                    let max_y = if axis == Axis::Y {
                        camera_section_center.get_y() > origin_y
                    } else {
                        camera_section_center.get_y() < origin_y
                    };
                    let max_z = if axis == Axis::Z {
                        camera_section_center.get_z() > origin_z
                    } else {
                        camera_section_center.get_z() < origin_z
                    };
                    let mut check_pos = Vector3d::from_xyz(
                        f64::from(origin_x + if max_x { 16 } else { 0 }),
                        f64::from(origin_y + if max_y { 16 } else { 0 }),
                        f64::from(origin_z + if max_z { 16 } else { 0 }),
                    );
                    let mut step = Vector3d::from_xyz(camera_pos.x, camera_pos.y, camera_pos.z);
                    step.sub(&check_pos)
                        .normalize()
                        .mul_scalar(SectionOcclusionGraph::ceiled_section_diagonal());
                    let mut visible = true;
                    while check_pos.distance_squared_xyz(camera_pos.x, camera_pos.y, camera_pos.z)
                        > 3600.0
                    {
                        check_pos.add(&step);
                        if check_pos.y > f64::from(view_area.max_y())
                            || check_pos.y < f64::from(view_area.min_y())
                        {
                            break;
                        }

                        let check_section = view_area.get_render_section_at(
                            &BlockPos::containing_xyz(check_pos.x, check_pos.y, check_pos.z),
                        );
                        if check_section.is_none_or(|check_section| {
                            storage.section_to_node_map[check_section].is_none()
                        }) {
                            visible = false;
                            break;
                        }
                    }

                    if !visible {
                        continue;
                    }
                }

                match &mut storage.section_to_node_map[render_section_at] {
                    Some(existing_node) => existing_node.add_source_direction(direction),
                    None => {
                        let mut new_node =
                            Node::new(render_section_at, Some(direction), node.step + 1);
                        new_node.set_directions(node.directions, direction);
                        queue.push_back(render_section_at);
                        storage.section_to_node_map[render_section_at] = Some(new_node);
                    }
                }
            }
        }
    }

    /// `ChunkPos.fromSectionNode`: the section's column, packed like `ChunkPos.pack`.
    fn chunk_node(section_node: i64) -> i64 {
        let x = SectionPos::x_of(section_node);
        let z = SectionPos::z_of(section_node);
        i64::from(x) & 0xffffffff | (i64::from(z) & 0xffffffff) << 32
    }

    fn offset_frustum(frustum: &Frustum) -> Frustum {
        let mut offset = frustum.clone();
        offset.offset_to_fully_include_camera_cube(8);
        offset
    }

    fn is_in_view_distance(
        &self,
        view_area: &ViewArea,
        camera_section_node: i64,
        section_node: i64,
    ) -> bool {
        SectionOcclusionGraph::is_in_view_distance_of(view_area, camera_section_node, section_node)
    }

    /// `ChunkTrackingView.isInViewDistance`.
    fn is_in_view_distance_of(
        view_area: &ViewArea,
        camera_section_node: i64,
        section_node: i64,
    ) -> bool {
        let view_distance = view_area.get_view_distance();
        let delta_x = i64::from(0.max(
            (SectionPos::x_of(section_node) - SectionPos::x_of(camera_section_node)).abs() - 1,
        ));
        let delta_z = i64::from(0.max(
            (SectionPos::z_of(section_node) - SectionPos::z_of(camera_section_node)).abs() - 1,
        ));
        let distance_squared = delta_x * delta_x + delta_z * delta_z;
        distance_squared < i64::from(view_distance * view_distance)
    }

    fn get_relative_from(
        view_area: &ViewArea,
        camera_section_node: i64,
        render_section: &crate::net::minecraft::client::renderer::chunk::section_render_dispatcher::RenderSection,
        direction: Direction,
    ) -> Option<usize> {
        let relative = render_section.get_neighbor_section_node(direction);
        let too_far = !SectionOcclusionGraph::is_in_view_distance_of(
            view_area,
            camera_section_node,
            relative,
        ) || Mth::abs_i32(
            SectionPos::y_of(camera_section_node) - SectionPos::y_of(relative),
        ) > view_area.get_view_distance();
        if too_far {
            None
        } else {
            view_area.get_render_section(relative)
        }
    }
}
