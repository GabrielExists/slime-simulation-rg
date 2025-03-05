#![cfg_attr(target_arch = "spirv", no_std)]

extern crate spirv_std;
extern crate core;

mod lerp_test;

use core::f32::consts::PI;
use glam::{Vec2, Vec4, vec2, UVec3, vec4};
use shared::*;
use spirv_std::{glam, spirv};
use spirv_std::glam::{IVec2, ivec2, UVec2, uvec2};
// Note: This cfg is incorrect on its surface, it really should be "are we compiling with std", but
// we tie #[no_std] above to the same condition, so it's fine.
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;
use spirv_std::num_traits::Pow;

enum Bounds {
    InsideBounds,
    OutsideBounds,
}

#[spirv(compute(threads(256, 1, 1)))]
pub fn main_cs(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(push_constant)] constants: &ShaderConstants,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] agents_buffer: &mut [Agent],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] agent_stats_buffer: &[AgentStats],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] trail_buffer: &mut [u32],
) {
    let agent_index = id.x as usize + id.y as usize * 256 + id.z as usize * 256 * 256;
    if agent_index >= agents_buffer.len() {
        return;
    }
    let agent = &mut agents_buffer[agent_index];
    if agent.channel_index as usize >= NUM_AGENT_TYPES {
        return;
    }
    let agent_stats = &agent_stats_buffer[agent.channel_index as usize];
    let random = hash((agent.y * constants.width as f32 + agent.x) as u32 + hash(id.x));

    // Sensor based on sensory data
    let weight_forward = sense(trail_buffer, constants, agent, &agent_stats, 0.0);
    let weight_left = sense(trail_buffer, constants, agent, &agent_stats, agent_stats.sensor_angle_spacing * PI / 180.0);
    let weight_right = sense(trail_buffer, constants, agent, &agent_stats, -agent_stats.sensor_angle_spacing * PI / 180.0);

    let random_steer_strength = random as f32 / u32::MAX as f32;
    let turn_speed = agent_stats.turn_speed * PI;
    let turn_speed_avoidance = agent_stats.turn_speed_avoidance * PI;

    match (weight_left, weight_forward, weight_right) {
        (None, _, Some(_)) => {
            agent.angle -= random_steer_strength * turn_speed_avoidance * constants.delta_time;
        }
        (Some(_), _, None) => {
            agent.angle += random_steer_strength * turn_speed_avoidance * constants.delta_time;
        }
        (Some(_), None, Some(_)) => {
            agent.angle += (random_steer_strength - 0.5) * 2.0 * turn_speed_avoidance * constants.delta_time;
        }
        (None, _, None) => {
            agent.angle += 0.0;
        }
        (Some(weight_left), Some(weight_forward), Some(weight_right)) => {
            // If center is stronger than edges, continue forward
            if weight_forward > weight_left && weight_forward > weight_right {
                agent.angle += 0.0;
            }
            // If edges are stronger than center, pick a direction randomly
            else if weight_left > weight_forward && weight_right > weight_forward {
                agent.angle += (random_steer_strength - 0.5) * 2.0 * turn_speed * constants.delta_time;
            }
            // If there's a gradient in one direction, turn that way
            else if weight_right > weight_left {
                agent.angle -= random_steer_strength * turn_speed * constants.delta_time;
            } else if weight_left > weight_right {
                agent.angle += random_steer_strength * turn_speed * constants.delta_time;
            }
        }
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
            let bounds = set_pixel(trail_buffer, constants, &agent_stats, ivec2(step_x as i32, step_y as i32), agent.channel_index as usize);
            if let Bounds::OutsideBounds = bounds {
                break 'clamp_block bounds;
            }
            num_steps -= step;
        }
        // If we didn't go outside the window, which is the normal case,
        // num_steps is now smaller than 1.0
        // Do the last little leap
        let previous_x = step_x;
        let previous_y = step_y;
        step_x = step_x + agent.angle.cos() * num_steps;
        step_y = step_y + agent.angle.sin() * num_steps;
        if previous_x as i32 != step_x as i32 || previous_y as i32 != step_y as i32 {
            let bounds = set_pixel(trail_buffer, constants, &agent_stats, ivec2(step_x as i32, step_y as i32), agent.channel_index as usize);
            break 'clamp_block bounds;
        }
        break 'clamp_block if is_inside_bounds(ivec2(step_x as i32, step_y as i32), constants) {
            Bounds::InsideBounds
        } else {
            Bounds::OutsideBounds
        };
    };
    if let Bounds::OutsideBounds = bounds {
        step_x = f32::min(constants.width as f32 - 1.01, f32::max(0.0, step_x));
        step_y = f32::min(constants.height as f32 - 1.01, f32::max(0.0, step_y));
        agent.angle = (random as f32 / u32::MAX as f32) * 2.0 * PI;
    }

    agent.x = step_x;
    agent.y = step_y;
}

fn sense(trail_buffer: &mut [u32], constants: &ShaderConstants, agent: &Agent, agent_stats: &AgentStats, angle_offset: f32) -> Option<f32> {
    let sensor_angle = agent.angle + angle_offset;
    let sensor_center = ivec2(
        (agent.x + sensor_angle.cos() * agent_stats.sensor_offset) as i32,
        (agent.y + sensor_angle.sin() * agent_stats.sensor_offset) as i32,
    );
    let mut sum = 0.0;

    for offset_x in -1..=1 {
        for offset_y in -1..=1 {
            let pos = sensor_center + ivec2(offset_x, offset_y);

            if is_inside_bounds(pos, constants) {
                let pixel = get_pixel(trail_buffer, constants, pos.as_uvec2());
                sum += pixel.get_frac(0) * agent_stats.attraction_channel_one;
                sum += pixel.get_frac(1) * agent_stats.attraction_channel_two;
                sum += pixel.get_frac(2) * agent_stats.attraction_channel_three;
                sum += pixel.get_frac(3) * agent_stats.attraction_channel_four;
            }
        }
    }
    if sum > agent_stats.avoidance_threshold {
        None
    } else {
        Some(sum)
    }
}


fn set_pixel(trail_buffer: &mut [u32], constants: &ShaderConstants, agent_stats: &AgentStats, position: IVec2, channel_index: usize) -> Bounds {
    if is_inside_bounds(position, constants) {
        let mut pixel = get_pixel(trail_buffer, constants, position.as_uvec2());
        let mut value_frac = pixel.get_frac(channel_index) as f32;
        value_frac += agent_stats.pixel_addition;
        pixel.set_frac(channel_index, f32::min(value_frac, 1.0));
        Bounds::InsideBounds
    } else {
        Bounds::OutsideBounds
    }
}

/// rust-gpu does not support returning a reference in an option it seems.
/// As a workaround, it is the callers responsibility to make sure position is within bounds
pub fn get_pixel<'pixel>(trail_buffer: &'pixel mut [u32], constants: &ShaderConstants, position: UVec2) -> PixelView<'pixel> {
    // if is_inside_bounds(position, constants) {
    let pixel_index = (position.y as usize * constants.width as usize + position.x as usize) * 2;
    // Some(
    // Safety: Safe since we're mutably borrowing different indices, which means there's not aliasing of mutable references
    unsafe {
        let a = &mut trail_buffer[pixel_index] as *mut _;
        let b = &mut trail_buffer[pixel_index + 1] as *mut _;
        PixelView::new(&mut *a, &mut *b)
    }
    // } else {
    //     None
    // }
}

fn is_inside_bounds(position: IVec2, constants: &ShaderConstants) -> bool {
    position.x >= 0 && position.x < constants.width as i32 && position.y >= 0 && position.y < constants.height as i32
}

fn is_inside_bounds_u(position: UVec2, constants: &ShaderConstants) -> bool {
    position.x < constants.width as u32 && position.y < constants.height as u32
}


#[spirv(compute(threads(8, 8, 4)))]
pub fn diffuse_cs(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(push_constant)] constants: &ShaderConstants,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] trail_stats: &[TrailStats],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] trail_buffer: &mut [u32],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] output_buffer: &mut [u32],
) {
    let pos = uvec2(id.x, id.y);
    if !is_inside_bounds_u(pos, constants) {
        return;
    }
    let channel_index = id.z as usize;
    if channel_index >= NUM_TRAIL_STATS {
        return;
    }
    let diffusion_speed = trail_stats[channel_index].diffusion_speed;
    let evaporation_speed = trail_stats[channel_index].evaporation_speed;

    let painting: Option<f32> = match (constants.mouse_down, constants.click_mode.decode()) {
        (0, _) => {
            None
        }
        (_, ClickMode::PaintTrail(paint_channel)) => {
            if paint_channel == channel_index as u32 {
                Some(1.0)
            } else {
                None
            }
        }
        (_, ClickMode::ResetTrail(reset_channel)) => {
            if reset_channel == channel_index as u32 {
                Some(0.0)
            } else {
                None
            }
        }
        (_, ClickMode::ResetAllTrails) => {
            Some(0.0)
        }
        _ => None
    };
    if let Some(paint_target) = painting {
        if within_range(pos.as_vec2(), constants.mouse_position, 5.0) {
            let mut output_pixel = get_pixel(output_buffer, constants, pos);
            output_pixel.set_frac(channel_index, paint_target);
            return;
        }
    }
    let mut sum = 0.0;
    for offset_x in -1..=1 {
        for offset_y in -1..=1 {
            let sample_pos = pos.as_ivec2() + ivec2(offset_x, offset_y);
            if is_inside_bounds(sample_pos, constants) {
                let pixel = get_pixel(trail_buffer, constants, sample_pos.as_uvec2());
                sum += pixel.get_frac(channel_index);
            }
        }
    }

    let mut output_pixel = get_pixel(output_buffer, constants, pos);
    let previous_value = output_pixel.get_frac(channel_index);
    let blur_result = sum / 9.0;
    let diffused_value = lerp(
        previous_value,
        blur_result,
        (diffusion_speed / 100.0) * constants.delta_time,
    );

    let evaporation_this_tick = (evaporation_speed / 100.0) * constants.delta_time;
    let new_value = f32::max(0.0, diffused_value - evaporation_this_tick);
    output_pixel.set_frac(channel_index, new_value);
}

fn within_range(first: Vec2, second: Vec2, distance: f32) -> bool {
    let square_distance = f32::sqrt((first.x - second.x).pow(2) + (first.y - second.y).pow(2));
    square_distance < distance.pow(2)
}


#[spirv(fragment)]
pub fn main_fs(
    #[spirv(frag_coord)] in_frag_coord: Vec4,
    #[spirv(push_constant)] constants: &ShaderConstants,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] trail_buffer: &mut [u32],
    output: &mut Vec4,
) {
    let position = ivec2(in_frag_coord.x as i32, in_frag_coord.y as i32);
    if is_inside_bounds(position, constants) {
        let pixel = get_pixel(trail_buffer, constants, position.as_uvec2());
        let r = f32::min(pixel.x_frac() + pixel.w_frac() * 0.3, 1.0);
        let g = f32::min(pixel.y_frac() + pixel.w_frac() * 0.3, 1.0);
        let b = f32::min(pixel.z_frac() + pixel.w_frac() * 0.3, 1.0);
        *output = vec4(r, g, b, 1.0)
        // let value = pixel.w_frac();
        // *output = vec4(value, value, value, 1.0)
    } else {
        *output = vec4(1.0, 1.0, 1.0, 1.0);
    }
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
