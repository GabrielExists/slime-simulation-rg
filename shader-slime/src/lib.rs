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

enum Bounds {
    InsideBounds,
    OutsideBounds,
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
    let agent_stats = &constants.agent_stats[0];
    let random = hash((agent.y * constants.width as f32 + agent.x) as u32 + hash(id.x));

    // Sensor based on sensory data
    let weight_forward = sense(trail_buffer, constants, agent, &agent_stats, 0.0);
    let weight_left = sense(trail_buffer, constants, agent, &agent_stats, agent_stats.sensor_angle_spacing);
    let weight_right = sense(trail_buffer, constants, agent, &agent_stats, -agent_stats.sensor_angle_spacing);

    let random_steer_strength = random as f32 / u32::MAX as f32;
    // If center is stronger than edges, continue forward
    if weight_forward > weight_left && weight_forward > weight_right {
        agent.angle += 0.0;
    }
    // If edges are stronger than center, pick a direction randomly
    else if weight_forward > weight_left && weight_forward > weight_right {
        agent.angle += (random_steer_strength - 0.5) * 2.0 * agent_stats.turn_speed * constants.delta_time;
    }
    // If there's a gradient in one direction, turn that way
    else if weight_right > weight_left {
        agent.angle -= random_steer_strength * agent_stats.turn_speed * constants.delta_time;
    }
    else if weight_left > weight_right {
        agent.angle -= random_steer_strength * agent_stats.turn_speed * constants.delta_time;
    }

    // Render each pixel inbetween here and the end of the streak we move this frame
    let mut num_steps = agent_stats.velocity * constants.delta_time;
    let step = 1.0;
    let mut step_x = agent.x;
    let mut step_y = agent.y;
    let bounds: Bounds = 'clamp_block: loop {
        while num_steps > step {
            step_x = step_x + agent.angle.cos() * step;
            step_y = step_y + agent.angle.sin() * step;
            let bounds = set_pixel(trail_buffer, constants, &vec2(step_x, step_y));
            if let Bounds::OutsideBounds = bounds {
                break 'clamp_block bounds;
            }
            num_steps -= step;
        }
        // If we didn't go outside the window, which is the normal case,
        // num_steps is now smaller than 1.0
        // Do the last little leap
        step_x = step_x + agent.angle.cos() * num_steps;
        step_y = step_y + agent.angle.sin() * num_steps;
        let bounds = set_pixel(trail_buffer, constants, &vec2(step_x, step_y));
        if let Bounds::OutsideBounds = bounds {
            break 'clamp_block bounds;
        }
        break 'clamp_block Bounds::InsideBounds;
    };
    if let Bounds::OutsideBounds = bounds {
        step_x = f32::min(constants.width as f32 - 0.01, f32::max(0.0, step_x));
        step_y = f32::min(constants.height as f32 - 0.01, f32::max(0.0, step_y));
        agent.angle = (random as f32 / u32::MAX as f32) * 2.0 * PI;
    }

    agent.x = step_x;
    agent.y = step_y;

}

fn sense(trail_buffer: &mut [u32], constants: &ShaderConstants, agent: &Agent, agent_stats: &AgentStats, angle_offset: f32) -> f32 {
    let sensor_angle = agent.angle + angle_offset;
    let sensor_center_x = agent.x + sensor_angle.cos() * agent_stats.sensor_offset;
    let sensor_center_y = agent.y + sensor_angle.sin() * agent_stats.sensor_offset;
    let mut sum = 0.0;

    for offset_x in -1..=1 {
        for offset_y in -1..=1 {
            let pos_x = sensor_center_x + offset_x as f32;
            let pos_y = sensor_center_y + offset_y as f32;

            if is_inside_bounds(&vec2(pos_x, pos_y), constants) {
                let index = pos_x as usize * constants.width as usize + pos_y as usize;
                sum += trail_buffer[index] as f32 / u32::MAX as f32;
            }
        }
    }
    return sum
}


fn set_pixel(trail_buffer: &mut [u32], constants: &ShaderConstants, position: &Vec2) -> Bounds {
    if is_inside_bounds(position, constants) {
        let pixel_index = position.y as usize * constants.width as usize + position.x as usize;
        trail_buffer[pixel_index] = 0xFFFFFFFF;
        Bounds::InsideBounds
    } else {
        Bounds::OutsideBounds
    }
}

fn is_inside_bounds(position: &Vec2, constants: &ShaderConstants) -> bool {
    position.x > 0.0 && position.x < constants.width as f32 && position.y > 0.0 && position.y < constants.height as f32
}


#[spirv(compute(threads(16, 16)))]
pub fn diffuse_cs(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(push_constant)] constants: &ShaderConstants,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] trail_buffer: &mut [u32],
) {
    let index = id.y as usize * constants.width as usize + id.x as usize;
    let original = trail_buffer[index];

    let mut sum = 0.0;
    for offset_x in -1..=1 {
        for offset_y in -1..=1 {
            let sample_x = id.x as i32 + offset_x;
            let sample_y = id.y as i32 + offset_y;
            if sample_x >= 0 && sample_x < constants.width as i32 && sample_y >= 0 && sample_y < constants.height as i32 {
                let sample_index: usize = sample_y as usize * constants.width as usize + sample_x as usize;
                sum += trail_buffer[sample_index] as f32 / u32::MAX as f32;
            }
        }
    }
    let blur_result = sum / 9.0;
    let diffused_value = lerp(
        original as f32 / u32::MAX as f32,
        blur_result,
        (constants.diffuse_speed / 100.0) * constants.delta_time
    );
    let diffused_upscaled = diffused_value * u32::MAX as f32;
    // trail_buffer[index] = (diffused_value * u32::MAX as f32) as u32;


    let evaporation_this_tick = (constants.evaporate_speed * u32::MAX as f32 / 100.0) * constants.delta_time;
    let new_value = f32::max(0.0, diffused_upscaled - evaporation_this_tick) as u32;
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

fn lerp(from: f32, to: f32, interpolation: f32) -> f32 {
    from + (to - from) * interpolation
}

fn hash(mut state: u32) -> u32 {
    state ^= 2747636419u32;
    state *= 2654435769u32;
    state ^= state >> 16;
    state *= 2654435769u32;
    state ^= state >> 16;
    state *= 2654435769u32;
    state
}

