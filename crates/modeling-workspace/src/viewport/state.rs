use super::camera::Camera;

#[derive(Default, Clone, Debug, PartialEq)]
pub struct ViewportState {
    pub l_button_pressed: bool,
    pub r_button_pressed: bool,
    pub cursor_position: iced::Point,
    pub camera: Camera,
}
