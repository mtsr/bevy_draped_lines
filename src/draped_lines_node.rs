use bevy::render::{
    render_graph::{CommandQueue, Node, ResourceSlots, SystemNode},
    renderer::{
        BufferId, BufferInfo, BufferMapMode, BufferUsage, RenderContext, RenderResourceBinding,
        RenderResourceBindings, RenderResourceContext,
    },
};
use bevy::transform::components::GlobalTransform;
use bevy::{
    core::{bytes_of, Pod, Zeroable},
    math::Vec4,
};
use bevy::{
    ecs::{
        prelude::{Local, Query, Res, ResMut, World},
        system::BoxedSystem,
        system::IntoSystem,
    },
    prelude::Color,
};

use crate::draped_line::{DrapedLine, DrapedLineUniform};

use super::uniform;

/// A Render Graph [Node] that write light data from the ECS to GPU buffers
#[derive(Debug, Default)]
pub struct DrapedLinesNode {
    command_queue: CommandQueue,
    max_draped_lines: usize,
}

impl DrapedLinesNode {
    pub fn new(max_draped_lines: usize) -> Self {
        DrapedLinesNode {
            max_draped_lines,
            command_queue: CommandQueue::default(),
        }
    }
}

impl Node for DrapedLinesNode {
    fn update(
        &mut self,
        _world: &World,
        render_context: &mut dyn RenderContext,
        _input: &ResourceSlots,
        _output: &mut ResourceSlots,
    ) {
        self.command_queue.execute(render_context);
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct DrapedLineCount {
    // storing as a `[u32; 4]` for memory alignement
    pub num_draped_lines: [u32; 4],
}

impl SystemNode for DrapedLinesNode {
    fn get_system(&self) -> BoxedSystem {
        let system = draped_lines_node_system.system().config(|config| {
            config.0 = Some(DrapedLinesNodeSystemState {
                command_queue: self.command_queue.clone(),
                max_draped_lines: self.max_draped_lines,
                draped_lines_buffer: None,
                staging_buffer: None,
            })
        });
        Box::new(system)
    }
}

/// Local "DrapedLines node system" state
#[derive(Debug, Default)]
pub struct DrapedLinesNodeSystemState {
    draped_lines_buffer: Option<BufferId>,
    staging_buffer: Option<BufferId>,
    command_queue: CommandQueue,
    max_draped_lines: usize,
}

pub fn draped_lines_node_system(
    mut state: Local<DrapedLinesNodeSystemState>,
    render_resource_context: Res<Box<dyn RenderResourceContext>>,
    mut render_resource_bindings: ResMut<RenderResourceBindings>,
    draped_lines: Query<(&DrapedLine, &GlobalTransform)>,
) {
    let state = &mut state;
    let render_resource_context = &**render_resource_context;

    let draped_line_count = draped_lines.iter().len().min(state.max_draped_lines);
    let draped_line_size = std::mem::size_of::<DrapedLineUniform>();
    let draped_line_array_size = draped_line_size * draped_line_count;
    let draped_line_array_max_size = draped_line_size * state.max_draped_lines;

    let draped_line_count_size = std::mem::size_of::<DrapedLineCount>();

    let draped_line_uniform_start = draped_line_count_size;
    let draped_line_uniform_end = draped_line_count_size + draped_line_array_size;

    let max_draped_line_uniform_size = draped_line_count_size + draped_line_array_max_size;

    if let Some(staging_buffer) = state.staging_buffer {
        if draped_line_count == 0 {
            return;
        }

        render_resource_context.map_buffer(staging_buffer, BufferMapMode::Write);
    } else {
        let buffer = render_resource_context.create_buffer(BufferInfo {
            size: max_draped_line_uniform_size,
            buffer_usage: BufferUsage::UNIFORM | BufferUsage::COPY_SRC | BufferUsage::COPY_DST,
            ..Default::default()
        });
        render_resource_bindings.set(
            uniform::DRAPED_LINES,
            RenderResourceBinding::Buffer {
                buffer,
                range: 0..max_draped_line_uniform_size as u64,
                dynamic_index: None,
            },
        );
        state.draped_lines_buffer = Some(buffer);

        let staging_buffer = render_resource_context.create_buffer(BufferInfo {
            size: max_draped_line_uniform_size,
            buffer_usage: BufferUsage::COPY_SRC | BufferUsage::MAP_WRITE,
            mapped_at_creation: true,
        });
        state.staging_buffer = Some(staging_buffer);
    }

    let staging_buffer = state.staging_buffer.unwrap();
    render_resource_context.write_mapped_buffer(
        staging_buffer,
        0..max_draped_line_uniform_size as u64,
        &mut |data, _renderer| {
            // DrapedLine count
            data[0..draped_line_count_size].copy_from_slice(bytes_of(&[
                draped_line_count as u32,
                0,
                0,
                0,
            ]));

            // point light array
            for ((draped_line, global_transform), slot) in draped_lines.iter().zip(
                data[draped_line_uniform_start..draped_line_uniform_end]
                    .chunks_exact_mut(draped_line_size),
            ) {
                slot.copy_from_slice(bytes_of(&DrapedLineUniform {
                    point0: global_transform.compute_matrix()
                        * Vec4::new(
                            draped_line.point0.x,
                            draped_line.point0.y,
                            draped_line.point0.z,
                            1.0,
                        ),
                    point1: global_transform.compute_matrix()
                        * Vec4::new(
                            draped_line.point1.x,
                            draped_line.point1.y,
                            draped_line.point1.z,
                            1.0,
                        ),
                    width: [draped_line.width, 0.0, 0.0, 0.0],
                    color: Color::RED.into(),
                    plane_dir: Vec4::new(
                        draped_line.plane_dir.x,
                        draped_line.plane_dir.y,
                        draped_line.plane_dir.z,
                        0.0,
                    ),
                }));
            }
        },
    );
    render_resource_context.unmap_buffer(staging_buffer);
    let draped_lines_buffer = state.draped_lines_buffer.unwrap();
    state.command_queue.copy_buffer_to_buffer(
        staging_buffer,
        0,
        draped_lines_buffer,
        0,
        max_draped_line_uniform_size as u64,
    );
}
