#![cfg_attr(target_arch = "spirv", no_std)]

mod lerp_test;

use core::f32::consts::PI;
use glam::{Vec2, Vec4, vec2, UVec3, vec4};
use shared::*;
use spirv_std::{glam, spirv};
// Note: This cfg is incorrect on its surface, it really should be "are we compiling with std", but
// we tie #[no_std] above to the same condition, so it's fine.
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;


fn hash(mut state: u32) -> u32 {
    state ^= 2747636419u32;
    state *= 2654435769u32;
    state ^= state >> 16;
    state *= 2654435769u32;
    state ^= state >> 16;
    state *= 2654435769u32;
    state
}

#[spirv(compute(threads(16)))]
pub fn main_cs(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(push_constant)] constants: &ShaderConstants,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] agents_buffer: &mut [Agent],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] trail_buffer: &mut [u32],
) {
    let agent_index = id.x as usize;
    let agent = &mut agents_buffer[agent_index];
    let random = hash((agent.y * constants.width as f32 + agent.x) as u32 + hash(id.x));

    let mut new_x = agent.x + agent.angle.cos() * constants.agent_stats[0].velocity * constants.delta_time;
    let mut new_y = agent.y + agent.angle.sin() * constants.agent_stats[0].velocity * constants.delta_time;
    if new_x < 0.0 || new_x > constants.width as f32 || new_y < 0.0 || new_y > constants.height as f32 {
        new_x = f32::min(constants.width as f32 - 0.01, f32::max(0.0, new_x));
        new_y = f32::min(constants.height as f32 - 0.01, f32::max(0.0, new_y));
        agent.angle = (random as f32 / u32::MAX as f32) * 2.0 * PI;
    }
    agent.x = new_x;
    agent.y = new_y;

    if agent.x < constants.width as f32 && agent.y < constants.height as f32 {
        let pixel_index = agent.y as usize * constants.width as usize + agent.x as usize;
        trail_buffer[pixel_index] = 0xFFFFFFFF;
    }
}

#[spirv(compute(threads(16, 16)))]
pub fn diffuse_cs(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(push_constant)] constants: &ShaderConstants,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] trail_buffer: &mut [u32],
) {
    let index = id.y as usize * constants.width as usize + id.x as usize;
    let original = trail_buffer[index];
    let evaporation_this_tick = (constants.evaporate_speed * u32::MAX as f32 / 100.0) * constants.delta_time;
    let new_value = f32::max(0.0, original as f32 - evaporation_this_tick) as u32;
    trail_buffer[index] = new_value;
}

#[spirv(fragment)]
pub fn main_fs(
    #[spirv(frag_coord)] in_frag_coord: Vec4,
    #[spirv(push_constant)] constants: &ShaderConstants,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] trail_buffer: &mut [u32],
    output: &mut Vec4,
) {
    if in_frag_coord.x as u32 >= constants.width || in_frag_coord.y as u32 >= constants.height {
        *output = vec4(1.0, 1.0, 1.0, 1.0);
        return;
    }
    // let frag_coord = vec2(in_frag_coord.x, in_frag_coord.y);
    // *output = fs(constants, frag_coord, sun_intensity_extra_spec_const_factor);
    let index = in_frag_coord.y as usize * constants.width as usize + in_frag_coord.x as usize;
    let pixel = trail_buffer[index];
    let normalized_pixel = pixel as f32 / u32::MAX as f32;
    *output = vec4(normalized_pixel, normalized_pixel, normalized_pixel, 1.0);
}

#[spirv(vertex)]
pub fn main_vs(#[spirv(vertex_index)] vert_idx: i32, #[spirv(position)] builtin_pos: &mut Vec4) {
    // Create a "full screen triangle" by mapping the vertex index.
    // ported from https://www.saschawillems.de/blog/2016/08/13/vulkan-tutorial-on-rendering-a-fullscreen-quad-without-buffers/
    let uv = vec2(((vert_idx << 1) & 2) as f32, (vert_idx & 2) as f32);
    let pos = 2.0 * uv - Vec2::ONE;

    *builtin_pos = pos.extend(0.0).extend(1.0);
}
