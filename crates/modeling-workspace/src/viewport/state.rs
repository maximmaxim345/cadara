#[derive(Default, Clone, Debug)]
pub struct ViewportState {
    pub(super) l_button_pressed: bool,
    pub(super) cursor_position: iced::Point,
    pub(super) camera_offset: iced::Vector<f32>,
}

#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Uniforms {
    pub(super) x: f32,
    pub(super) y: f32,
}
