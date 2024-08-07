#[derive(Default, Clone, Debug)]
pub struct ViewportState {
    pub l_button_pressed: bool,
    pub cursor_position: iced::Point,
    pub camera_pos: glam::Vec2,
}

#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct CameraUniform {
    pub view_pos: glam::Vec2,
}
