//! Ported to Rust from <https://github.com/Tw1ddle/Sky-Shader/blob/master/src/shaders/glsl/sky.fragment>

#![cfg_attr(target_arch = "spirv", no_std)]

use core::f32::consts::PI;
use glam::{Vec3, vec3};

use spirv_std::glam;

// Note: This cfg is incorrect on its surface, it really should be "are we compiling with std", but
// we tie #[no_std] above to the same condition, so it's fine.
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use bytemuck::{Pod, Zeroable};

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum SpawnMode {
    EvenlyDistributed,
    CenterFacingOutwards,
    PointFacingOutwards {
        x: f32,
        y: f32,
    },
    CircleFacingInwards {
        max_distance: f32,
    },
    CircumferenceFacingInward {
        distance: f32,
    },
    CircumferenceFacingOutward {
        distance: f32,
    },
    CircumferenceFacingRandom {
        distance: f32,
    },
    CircumferenceFacingClockwise {
        distance: f32,
    },
}


#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct ShaderConstants {
    pub width: u32,
    pub height: u32,
    pub time: f32,
    pub delta_time: f32,
    pub agent_stats: [AgentStats; NUM_AGENT_TYPES],
}

#[derive(Copy, Clone)]
pub struct AgentStatsAll {
    pub spawn_mode: SpawnMode,
    pub num_agents: usize,
    pub shader_stats: AgentStats,
}

pub const NUM_AGENT_TYPES: usize = 2;
#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct AgentStats {
    // Pixels travelled per second
    pub velocity: f32,
    pub turn_speed: f32,
    pub turn_speed_avoidance: f32,
    pub sensor_angle_spacing: f32,
    pub sensor_offset: f32,
    pub pixel_addition: f32,
    // Maximum value is 9.0
    // Minimum value is 0.0
    // Setting a value over 9 effectively disables avoidance of saturated trails
    pub avoidance_threshold: f32,
    // Percent of full white to black transition per second.
    // 100.0 is completely faded after 1 second.
    // 50.0 is completely faded after 2 seconds.
    pub evaporation_speed: f32,
    // Speed of diffusion in percent.
    // Reaching 90% takes 1 second if set to 240%.
    // Reaching 86% takes 1 second if set to 200%.
    // Reaching 63% takes 1 second if set to 100%.
    pub diffusion_speed: f32,
    pub attraction_blue: f32,
    pub attraction_green: f32,
}

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct Agent {
    pub x: f32,
    pub y: f32,
    pub angle: f32,
    pub channel_index: u32,
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

// fn vec_from_u32(storage: &mut u32) -> &mut UVec2 {
//     from_bytes_mut::<UVec2>(bytemuck::bytes_of_mut(storage))
// }

pub struct PixelView<'storage> {
    storage: &'storage mut u32,
}

pub fn pixel_view(storage: &mut u32) -> PixelView {
    PixelView::new(storage)
}

impl<'storage> PixelView<'storage> {
    pub fn new(storage: &'storage mut u32) -> Self {
        PixelView {
            storage,
        }
    }
    pub fn x(&self) -> u32 { *self.storage >> 16 }
    pub fn y(&self) -> u32 { *self.storage  & 0xFFFF }
    pub fn get(&self, index: usize) -> u32 {
        match index {
            0 => self.x(),
            _ => self.y(),
        }
    }
    pub fn x_frac(&self) -> f32 { frac_from_int(self.x()) }
    pub fn y_frac(&self) -> f32 { frac_from_int(self.y()) }
    pub fn get_frac(&self, index: usize) -> f32 {
        frac_from_int(self.get(index))
    }
    pub fn set_x(&mut self, value: u32) {
        *self.storage = *self.storage & 0x0000FFFF | value << 16;
    }
    pub fn set_y(&mut self, value: u32) {
        *self.storage = *self.storage & 0xFFFF0000 | (value & 0xFFFF);
    }
    pub fn set(&mut self, index: usize, value: u32) {
        match index {
            0 => self.set_x(value),
            _ => self.set_y(value),
        }
    }
    pub fn set_x_frac(&mut self, value: f32) {
        self.set_x(int_from_frac(value))
    }
    pub fn set_y_frac(&mut self, value: f32) {
        self.set_y(int_from_frac(value))
    }
    pub fn set_frac(&mut self, index: usize, value: f32) {
        match index {
            0 => self.set_x_frac(value),
            _ => self.set_y_frac(value),
        }
    }
}

// impl<'storage> PixelView<'storage> {
//     pub fn new(storage: &'storage mut u32) -> Self {
//         PixelView {
//             storage,
//         }
//     }
//     pub fn x(&self) -> u32 { (*self.storage >> 16) & 0xFFFF }
//     pub fn y(&self) -> u32 { (*self.storage >> 8) & 0xFF }
//     pub fn z(&self) -> u32 { *self.storage & 0xFF }
//     pub fn set_x(&mut self, value: u32) {
//         *self.storage = *self.storage & 0x0000FFFF | (value & 0xFFFF) << 16;
//     }
//     pub fn set_y(&mut self, value: u32) {
//         *self.storage = *self.storage & 0xFFFF00FF | (value & 0xFF) << 8;
//     }
//     pub fn set_z(&mut self, value: u32) {
//         *self.storage = *self.storage & 0xFFFFFF00 | (value & 0xFF);
//     }
//     pub fn x_frac(&self) -> f32 { self.x() as f32 / 255.0 }
//     pub fn y_frac(&self) -> f32 { self.y() as f32 / 255.0 }
//     pub fn z_frac(&self) -> f32 { self.z() as f32 / 255.0 }
//     pub fn set_x_frac(&mut self, value: f32) {
//         self.set_x(int_from_frac(value))
//     }
//     pub fn set_y_frac(&mut self, value: f32) {
//         self.set_y(int_from_frac(value))
//     }
//     pub fn set_z_frac(&mut self, value: f32) {
//         self.set_z(int_from_frac(value))
//     }
// }


pub const PIXEL_MAX: u32 = 65535;

pub fn frac_from_int(value: u32) -> f32 {
    value as f32 / PIXEL_MAX as f32
}

pub fn int_from_frac(value: f32) -> u32 {
    (value * PIXEL_MAX as f32) as u32
}

#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    fn test_set() {
        let mut a = 0x12345678;
        pixel_view(&mut a).set_x(0xAB);
        assert_eq!(a, 0xAB345678);
        pixel_view(&mut a).set_y(0xCD);
        assert_eq!(a, 0xABCD5678);
        pixel_view(&mut a).set_z(0xEF);
        assert_eq!(a, 0xABCDEF78);
        pixel_view(&mut a).set_w(0xBD);
        assert_eq!(a, 0xABCDEFBD);
    }
    #[test]
    fn test_get() {
        let mut a = 0x12345678;
        let view = pixel_view(&mut a);
        assert_eq!(view.x(), 0x12);
        assert_eq!(view.y(), 0x34);
        assert_eq!(view.z(), 0x56);
        assert_eq!(view.w(), 0x78);
    }

    #[test]
    fn test_set_index() {
        let mut a = 0x12345678;
        pixel_view(&mut a).set(0, 0xAB);
        assert_eq!(a, 0xAB345678);
        pixel_view(&mut a).set(1, 0xCD);
        assert_eq!(a, 0xABCD5678);
        pixel_view(&mut a).set(2, 0xEF);
        assert_eq!(a, 0xABCDEF78);
        pixel_view(&mut a).set(3, 0xBD);
        assert_eq!(a, 0xABCDEFBD);
    }
    #[test]
    fn test_get_index() {
        let mut a = 0x12345678;
        let view = pixel_view(&mut a);
        assert_eq!(view.get(0), 0x12);
        assert_eq!(view.get(1), 0x34);
        assert_eq!(view.get(2), 0x56);
        assert_eq!(view.get(3), 0x78);
    }
}
