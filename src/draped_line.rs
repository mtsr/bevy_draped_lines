use bevy::{
    core::{Pod, Zeroable},
    ecs::bundle::Bundle,
    math::{Vec3, Vec4},
    prelude::{Color, GlobalTransform, Transform},
};

#[derive(Bundle, Debug, Default)]
pub struct DrapedLineBundle {
    pub draped_line: DrapedLine,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

#[derive(Clone, Debug)]
pub struct DrapedLine {
    pub point0: Vec3,
    pub point1: Vec3,
    pub width: f32,
    pub color: Color,
    pub plane_dir: Vec3,
}

impl Default for DrapedLine {
    fn default() -> Self {
        DrapedLine {
            point0: Default::default(),
            point1: Default::default(),
            width: 1.0,
            color: Color::WHITE,
            plane_dir: -Vec3::Y,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct DrapedLineUniform {
    pub point0: Vec4,    // padding
    pub point1: Vec4,    // padding
    pub width: [f32; 4], // padding
    pub color: Vec4,
    pub plane_dir: Vec4, // padding
}
