//! Ported to Rust from <https://github.com/Tw1ddle/Sky-Shader/blob/master/src/shaders/glsl/sky.fragment>
#![cfg_attr(target_arch = "spirv", no_std)]

pub mod pixel_view;
use bytemuck::{Pod, Zeroable};
use core::f32::consts::PI;
use glam::{UVec2, Vec2, Vec3, Vec4, vec3};
use spirv_std::glam;

// Note: This cfg is incorrect on its surface, it really should be "are we compiling with std", but
// we tie #[no_std] above to the same condition, so it's fine.
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

#[cfg(not(target_arch = "spirv"))]
use std::fmt::{Display, Formatter};
// #[cfg(not(target_arch = "spirv"))]
// use std::fmt::format;
#[cfg(not(target_arch = "spirv"))]
use serde::de::{SeqAccess, Visitor};
#[cfg(not(target_arch = "spirv"))]
use serde::ser::SerializeSeq;
#[cfg(not(target_arch = "spirv"))]
use serde::{Deserialize, Serialize};
#[cfg(not(target_arch = "spirv"))]
use serde::{Deserializer, Serializer};

#[cfg_attr(not(target_arch = "spirv"), derive(Serialize, Deserialize))]
#[derive(Copy, Clone, PartialEq, Default)]
pub enum SpawnMode {
    #[default]
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
            SpawnMode::CircumferenceFacingInward { .. } => {
                f.write_str("Circumference facing inwards")
            }
            SpawnMode::CircumferenceFacingOutward { .. } => {
                f.write_str("Circumference facing outward")
            }
            SpawnMode::CircumferenceFacingRandom { .. } => {
                f.write_str("Circumference facing random")
            }
            SpawnMode::CircumferenceFacingClockwise { .. } => {
                f.write_str("Circumference facing clockwise")
            }
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
            _ => None,
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
            _ => ClickMode::Disabled,
        }
    }
}

#[cfg_attr(not(target_arch = "spirv"), derive(Serialize, Deserialize))]
#[derive(Copy, Clone, PartialEq)]
#[repr(C)]
pub enum ColorMode {
    Disabled,
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[cfg(not(target_arch = "spirv"))]
impl Display for ColorMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorMode::Disabled => f.write_str("Disabled"),
            ColorMode::Add => f.write_str("Add"),
            ColorMode::Subtract => f.write_str("Subtract"),
            ColorMode::Multiply => f.write_str("Multiply"),
            ColorMode::Divide => f.write_str("Divide"),
        }
    }
}

impl ColorMode {
    pub const fn encode(self) -> ColorModeEncoded {
        let number = match self {
            ColorMode::Add => 0,
            ColorMode::Subtract => 1,
            ColorMode::Multiply => 2,
            ColorMode::Divide => 3,
            _ => 0xFF,
        };
        ColorModeEncoded(number)
    }
}

#[cfg_attr(not(target_arch = "spirv"), derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Pod, Zeroable, PartialEq)]
#[repr(C)]
pub struct ColorModeEncoded(u32);

impl ColorModeEncoded {
    pub fn decode(self) -> ColorMode {
        match self.0 {
            0 => ColorMode::Add,
            1 => ColorMode::Subtract,
            2 => ColorMode::Multiply,
            3 => ColorMode::Divide,
            _ => ColorMode::Disabled,
        }
    }
}

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct ShaderConstants {
    pub screen_size: UVec2,
    pub map_size: UVec2,
    pub time: f32,
    pub time_step: f32,
    pub padding_1: f32,
    pub padding_2: f32,
    pub background_color: Color,
}

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct MouseConstants {
    pub screen_size: UVec2,
    pub map_size: UVec2,
    pub click_mode: ClickModeEncoded,
    pub mouse_down: u32,
    pub mouse_position: Vec2,
    pub last_mouse_position: Vec2,
    pub brush_size: f32,
    pub _padding: f32,
}

pub const NUM_AGENT_TYPES: usize = 4;

#[cfg_attr(not(target_arch = "spirv"), derive(Serialize, Deserialize))]
#[derive(Copy, Clone, PartialEq, Default, Pod, Zeroable)]
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
    pub timeout: f32,
    pub timeout_conversion: u32,
    pub interaction_channels: [TrailInteraction; NUM_TRAIL_STATS],
}

#[cfg_attr(not(target_arch = "spirv"), derive(Serialize, Deserialize))]
#[derive(Copy, Clone, PartialEq, Default, Pod, Zeroable)]
#[repr(C)]
pub struct TrailInteraction {
    pub attraction: f32,
    pub addition: f32,
    pub conversion_enabled: u32,
    pub conversion_threshold: f32,
    pub conversion: u32,
}

pub const NUM_TRAIL_STATS: usize = 4;

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
    pub padding_1: f32,
    pub color_mode: ColorModeEncoded,
    pub color: Color,
}

#[derive(Copy, Clone, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct Color {
    pub inner: Vec4,
}
impl Color {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color {
            inner: Vec4::new(r, g, b, a),
        }
    }
}
#[cfg(not(target_arch = "spirv"))]
impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(4))?;
        seq.serialize_element(&self.inner.x)?;
        seq.serialize_element(&self.inner.y)?;
        seq.serialize_element(&self.inner.z)?;
        seq.serialize_element(&self.inner.w)?;
        seq.end()
    }
}
#[cfg(not(target_arch = "spirv"))]
struct ColorVisitor;
#[cfg(not(target_arch = "spirv"))]
impl<'de> Visitor<'de> for ColorVisitor {
    type Value = Color;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("array of four floats")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        if let Some(x) = seq.next_element()? {
            if let Some(y) = seq.next_element()? {
                if let Some(z) = seq.next_element()? {
                    if let Some(w) = seq.next_element()? {
                        return Ok(Color {
                            inner: Vec4::new(x, y, z, w),
                        });
                    }
                }
            }
        }
        Err(serde::de::Error::custom(
            "missing items when deserializing color",
        ))
    }
}
#[cfg(not(target_arch = "spirv"))]
impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(ColorVisitor)
    }
}

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct Agent {
    pub x: f32,
    pub y: f32,
    pub angle: f32,
    pub agent_type: u32,
    pub countdown: f32,
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

#[cfg(test)]
mod test {
    use crate::*;

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

    #[test]
    fn test_color_mode_encoding() {
        for value in 0..u16::MAX as u32 {
            let reference = ColorModeEncoded(value);
            let click_mode = reference.decode();
            let encoded = click_mode.encode();
            match click_mode {
                ColorMode::Disabled => {}
                _ => {
                    assert_eq!(reference.0, encoded.0)
                }
            }
        }
    }
}
