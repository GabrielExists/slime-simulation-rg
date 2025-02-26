//! Ported to Rust from <https://github.com/Tw1ddle/Sky-Shader/blob/master/src/shaders/glsl/sky.fragment>

use bytemuck::{Pod, Zeroable};

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct ShaderConstants {
    pub width: u32,
    pub height: u32,
    pub time: f32,

    pub cursor_x: f32,
    pub cursor_y: f32,
    pub drag_start_x: f32,
    pub drag_start_y: f32,
    pub drag_end_x: f32,
    pub drag_end_y: f32,

    /// Bit mask of the pressed buttons (0 = Left, 1 = Middle, 2 = Right).
    pub mouse_button_pressed: u32,

    /// The last time each mouse button (Left, Middle or Right) was pressed,
    /// or `f32::NEG_INFINITY` for buttons which haven't been pressed yet.
    ///
    /// If this is the first frame after the press of some button, that button's
    /// entry in `mouse_button_press_time` will exactly equal `time`.
    pub mouse_button_press_time: [f32; 3],
}

