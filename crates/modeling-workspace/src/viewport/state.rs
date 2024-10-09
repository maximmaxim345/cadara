use iced::widget::shader::wgpu;

use super::camera::Camera;

#[derive(Default, Clone, Debug)]
pub struct ViewportState {
    pub l_button_pressed: bool,
    pub r_button_pressed: bool,
    pub cursor_position: iced::Point,
    pub camera: Camera,
}

#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Vertex {
    pub pos: glam::Vec3,
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![
        0 => Float32x3,
    ];

    pub const fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}
