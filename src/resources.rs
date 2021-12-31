#[derive(Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct PlayerInputState {
    pub up_pressed: bool,
    pub down_pressed: bool,
    pub right_pressed: bool,
    pub left_pressed: bool,
}
