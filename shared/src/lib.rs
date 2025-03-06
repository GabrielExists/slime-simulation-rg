//! Ported to Rust from <https://github.com/Tw1ddle/Sky-Shader/blob/master/src/shaders/glsl/sky.fragment>

#![cfg_attr(target_arch = "spirv", no_std)]

extern crate bytemuck;
extern crate spirv_std;

#[cfg(not(target_arch = "spirv"))]
use std::fmt::{Display, Formatter};
#[cfg(not(target_arch = "spirv"))]
use serde::{Serialize, Deserialize};

use core::f32::consts::PI;
use glam::{Vec3, vec3};

use spirv_std::glam;

// Note: This cfg is incorrect on its surface, it really should be "are we compiling with std", but
// we tie #[no_std] above to the same condition, so it's fine.
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use bytemuck::{Pod, Zeroable};
use spirv_std::glam::{UVec2, Vec2};

#[cfg_attr(not(target_arch = "spirv"), derive(Serialize, Deserialize))]
#[derive(Copy, Clone, PartialEq)]
pub enum SpawnMode {
    EvenlyDistributed,
    CenterFacingOutward,
    PointFacingOutward {
        x: u32,
        y: u32,
    },
    CircleFacingInward {
        max_distance: u32,
    },
    CircumferenceFacingInward {
        distance: u32,
    },
    CircumferenceFacingOutward {
        distance: u32,
    },
    CircumferenceFacingRandom {
        distance: u32,
    },
    CircumferenceFacingClockwise {
        distance: u32,
    },
    BoxFacingRandom {
        spawn_box: SpawnBox,
    },
}

#[cfg_attr(not(target_arch = "spirv"), derive(Serialize, Deserialize))]
#[derive(Copy, Clone, PartialEq)]
pub struct SpawnBox {
    pub left: u32,
    pub top: u32,
    pub box_width: u32,
    pub box_height: u32,
}

impl Default for SpawnBox {
    fn default() -> Self {
        Self {
            left: 100,
            top: 100,
            box_width: 300,
            box_height: 300,
        }
    }
}

#[cfg(not(target_arch = "spirv"))]
impl Display for SpawnMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SpawnMode::EvenlyDistributed => f.write_str("Evenly distributed"),
            SpawnMode::CenterFacingOutward => f.write_str("Center facing outward"),
            SpawnMode::PointFacingOutward { .. } => f.write_str("Point facing outward"),
            SpawnMode::CircleFacingInward { .. } => f.write_str("Circle facing inward"),
            SpawnMode::CircumferenceFacingInward { .. } => f.write_str("Circumference facing inwards"),
            SpawnMode::CircumferenceFacingOutward { .. } => f.write_str("Circumference facing outward"),
            SpawnMode::CircumferenceFacingRandom { .. } => f.write_str("Circumference facing random"),
            SpawnMode::CircumferenceFacingClockwise { .. } => f.write_str("Circumference facing clockwise"),
            SpawnMode::BoxFacingRandom { .. } => f.write_str("Box"),
        }
    }
}

impl SpawnMode {
    pub fn distance(&self) -> Option<u32> {
        match self {
            SpawnMode::EvenlyDistributed => None,
            SpawnMode::CenterFacingOutward => None,
            SpawnMode::PointFacingOutward { .. } => None,
            SpawnMode::CircleFacingInward { max_distance } => Some(*max_distance),
            SpawnMode::CircumferenceFacingInward { distance } => Some(*distance),
            SpawnMode::CircumferenceFacingOutward { distance } => Some(*distance),
            SpawnMode::CircumferenceFacingRandom { distance } => Some(*distance),
            SpawnMode::CircumferenceFacingClockwise { distance } => Some(*distance),
            SpawnMode::BoxFacingRandom { .. } => None,
        }
    }
    pub fn spawn_box(&self) -> Option<SpawnBox> {
        match self {
            SpawnMode::BoxFacingRandom { spawn_box } => Some(*spawn_box),
            _ => None
        }
    }
}

#[cfg_attr(not(target_arch = "spirv"), derive(Serialize, Deserialize))]
#[derive(Copy, Clone, PartialEq)]
#[repr(C)]
pub enum ClickMode {
    Disabled,
    ShowMenu,
    PaintTrail(u32),
    ResetTrail(u32),
    ResetAllTrails,
}

#[cfg(not(target_arch = "spirv"))]
impl Display for ClickMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ClickMode::Disabled => f.write_str("Disabled"),
            ClickMode::ShowMenu => f.write_str("Show menu"),
            ClickMode::PaintTrail(_) => f.write_str("Paint trail"),
            ClickMode::ResetTrail(_) => f.write_str("Reset trail"),
            ClickMode::ResetAllTrails => f.write_str("Reset all trails"),
        }
    }
}

impl ClickMode {
    pub fn encode(self) -> ClickModeEncoded {
        let number = match self {
            ClickMode::Disabled => 0,
            ClickMode::ShowMenu => 1,
            ClickMode::PaintTrail(trail_index) => 256 + trail_index,
            ClickMode::ResetTrail(trail_index) => 512 + trail_index,
            ClickMode::ResetAllTrails => 2,
        };
        ClickModeEncoded(number)
    }
}

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct ClickModeEncoded(u32);

impl ClickModeEncoded {
    pub fn decode(self) -> ClickMode {
        match self.0 {
            0 => ClickMode::Disabled,
            1 => ClickMode::ShowMenu,
            2 => ClickMode::ResetAllTrails,
            trail_index @ 256..=511 => ClickMode::PaintTrail(trail_index - 256),
            trail_index @ 512..=767 => ClickMode::ResetTrail(trail_index - 512),
            _ => ClickMode::Disabled
        }
    }
}

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct ShaderConstants {
    pub screen_size: UVec2,
    pub time: f32,
    pub delta_time: f32,
}

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct MouseConstants {
    pub screen_size: UVec2,
    pub click_mode: ClickModeEncoded,
    pub mouse_down: u32,
    pub mouse_position: Vec2,
    pub last_mouse_position: Vec2,
    pub brush_size: f32,
    pub _padding: f32,
}

pub const NUM_AGENT_TYPES: usize = 1;

#[cfg_attr(not(target_arch = "spirv"), derive(Serialize, Deserialize))]
#[derive(Copy, Clone, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct AgentStats {
    // Pixels travelled per second
    pub velocity: f32,
    pub turn_speed: f32,
    pub turn_speed_avoidance: f32,
    // Maximum value is 9.0
    // Minimum value is 0.0
    // Setting a value over 9 effectively disables avoidance of saturated trails
    pub avoidance_threshold: f32,
    pub sensor_angle_spacing: f32,
    pub sensor_offset: f32,
    pub interaction_channels: [TrailInteraction; NUM_TRAIL_STATS],
}
#[cfg_attr(not(target_arch = "spirv"), derive(Serialize, Deserialize))]
#[derive(Copy, Clone, PartialEq, Pod, Zeroable, Default)]
#[repr(C)]
pub struct TrailInteraction {
    pub attraction: f32,
    pub addition: f32,
    pub conversion_enabled: u32,
    pub conversion_threshold: f32,
    pub conversion: u32,
}

pub const NUM_TRAIL_STATS: usize = 4;
pub const INTS_PER_PIXEL: u32 = NUM_TRAIL_STATS.div_ceil(2) as u32;

#[cfg_attr(not(target_arch = "spirv"), derive(Serialize, Deserialize))]
#[derive(Copy, Clone, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct TrailStats {
    // Percent of full white to black transition per second.
    // 100.0 is completely faded after 1 second.
    // 50.0 is completely faded after 2 seconds.
    pub evaporation_speed: f32,
    // Speed of diffusion in percent.
    // Reaching 90% takes 1 second if set to 240%.
    // Reaching 86% takes 1 second if set to 200%.
    // Reaching 63% takes 1 second if set to 100%.
    pub diffusion_speed: f32,
}

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct Agent {
    pub x: f32,
    pub y: f32,
    pub angle: f32,
    pub agent_type: u32,
}

pub fn saturate(x: f32) -> f32 {
    x.clamp(0.0, 1.0)
}

pub fn pow(v: Vec3, power: f32) -> Vec3 {
    vec3(v.x.powf(power), v.y.powf(power), v.z.powf(power))
}

pub fn exp(v: Vec3) -> Vec3 {
    vec3(v.x.exp(), v.y.exp(), v.z.exp())
}

/// Based on: <https://seblagarde.wordpress.com/2014/12/01/inverse-trigonometric-functions-gpu-optimization-for-amd-gcn-architecture/>
pub fn acos_approx(v: f32) -> f32 {
    let x = v.abs();
    let mut res = -0.155972 * x + 1.56467; // p(x)
    res *= (1.0f32 - x).sqrt();

    if v >= 0.0 { res } else { PI - res }
}

pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    // Scale, bias and saturate x to 0..1 range
    let x = saturate((x - edge0) / (edge1 - edge0));
    // Evaluate polynomial
    x * x * (3.0 - 2.0 * x)
}

pub struct PixelView<'storage> {
    storage_one: &'storage mut u32,
    storage_two: &'storage mut u32,
}

impl<'storage> PixelView<'storage> {
    pub fn new(storage_one: &'storage mut u32, storage_two: &'storage mut u32) -> Self {
        PixelView {
            storage_one,
            storage_two,
        }
    }
    pub fn x(&self) -> u32 { *self.storage_one >> 0 & 0xFFFF }
    pub fn y(&self) -> u32 { *self.storage_one >> 16 & 0xFFFF }
    pub fn z(&self) -> u32 { *self.storage_two >> 0 & 0xFFFF }
    pub fn w(&self) -> u32 { *self.storage_two >> 16 & 0xFFFF }
    pub fn get(&self, index: usize) -> u32 {
        match index {
            0 => self.x(),
            1 => self.y(),
            2 => self.z(),
            _ => self.w(),
        }
    }
    pub fn x_frac(&self) -> f32 { frac_from_int(self.x()) }
    pub fn y_frac(&self) -> f32 { frac_from_int(self.y()) }
    pub fn z_frac(&self) -> f32 { frac_from_int(self.z()) }
    pub fn w_frac(&self) -> f32 { frac_from_int(self.w()) }
    pub fn get_frac(&self, index: usize) -> f32 {
        frac_from_int(self.get(index))
    }
    pub fn set_x(&mut self, value: u32) {
        *self.storage_one = *self.storage_one & 0xFFFF0000 | (value & 0xFFFF) << 0;
    }
    pub fn set_y(&mut self, value: u32) {
        *self.storage_one = *self.storage_one & 0x0000FFFF | ((value & 0xFFFF) << 16);
    }
    pub fn set_z(&mut self, value: u32) {
        *self.storage_two = *self.storage_two & 0xFFFF0000 | (value & 0xFFFF) << 0;
    }
    pub fn set_w(&mut self, value: u32) {
        *self.storage_two = *self.storage_two & 0x0000FFFF | ((value & 0xFFFF) << 16);
    }
    pub fn set(&mut self, index: usize, value: u32) {
        match index {
            0 => self.set_x(value),
            1 => self.set_y(value),
            2 => self.set_z(value),
            _ => self.set_w(value),
        }
    }
    pub fn set_x_frac(&mut self, value: f32) {
        self.set_x(int_from_frac(value))
    }
    pub fn set_y_frac(&mut self, value: f32) {
        self.set_y(int_from_frac(value))
    }
    pub fn set_z_frac(&mut self, value: f32) {
        self.set_z(int_from_frac(value))
    }
    pub fn set_w_frac(&mut self, value: f32) {
        self.set_w(int_from_frac(value))
    }
    pub fn set_frac(&mut self, index: usize, value: f32) {
        match index {
            0 => self.set_x_frac(value),
            1 => self.set_y_frac(value),
            2 => self.set_z_frac(value),
            _ => self.set_w_frac(value),
        }
    }
}



pub const PIXEL_MAX: u32 = 2u32.pow(15) - 1;

pub fn frac_from_int(value: u32) -> f32 {
    value as f32 / PIXEL_MAX as f32
}

pub fn int_from_frac(value: f32) -> u32 {
    if value > 0.999 {
        PIXEL_MAX
    } else {
        (value * PIXEL_MAX as f32) as u32
    }
}

#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    fn test_set() {
        let mut a = 0x12345678;
        let mut b = 0x98765432;
        let mut pixel = PixelView::new(&mut a, &mut b);
        pixel.set_x(0x2BCD);
        pixel.set_y(0x4DEF);
        pixel.set_z(0xFEDC);
        pixel.set_w(0xBA87);
        assert_eq!(pixel.x(), 0x2BCD);
        assert_eq!(pixel.y(), 0x4DEF);
        assert_eq!(pixel.z(), 0xFEDC);
        assert_eq!(pixel.w(), 0xBA87);
    }

    #[test]
    fn test_set_frac() {
        let mut a = 0x12345678;
        let mut b = 0x98765432;
        let mut pixel = PixelView::new(&mut a, &mut b);
        pixel.set_x_frac(0.5);
        pixel.set_y_frac(0.25);
        pixel.set_z_frac(0.75);
        pixel.set_w_frac(0.125);
        assert!(f32::abs(pixel.x_frac() - 0.5) < 0.01);
        assert!(f32::abs(pixel.y_frac() - 0.25) < 0.01);
        assert!(f32::abs(pixel.z_frac() - 0.75) < 0.01);
        assert!(f32::abs(pixel.w_frac() - 0.125) < 0.01);
    }

    #[test]
    fn test_click_mode_encoding() {
        for value in 0..u16::MAX as u32 {
            let reference = ClickModeEncoded(value);
            let click_mode = reference.decode();
            let encoded = click_mode.encode();
            match click_mode {
                ClickMode::Disabled => {}
                _ => {
                    assert_eq!(reference.0, encoded.0)
                }
            }
        }
    }
}
