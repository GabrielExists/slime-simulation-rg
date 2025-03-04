use std::thread;
use std::time::{Duration, Instant};
use winit::dpi::PhysicalSize;
use crate::configuration_menu::ConfigurationValues;
use wgpu::util::DeviceExt;
use shared::ShaderConstants;

/// A slot is the backend that a shader "slots" into.
/// Each shader has individual bindings and render steps that need to be matched in the cpu,
/// and by making the collection a trait we can collect them easily iterate over them
/// or disable one

// Data that is created at program init
pub struct ProgramInit<'window> {
    pub device: wgpu::Device,
    pub surface_format: wgpu::TextureFormat,
    pub module: wgpu::ShaderModule,
    pub queue: wgpu::Queue,
    pub window: &'window winit::window::Window,
}

// Data regenerated when window is resized
pub struct ProgramBuffers {
    pub screen_size: PhysicalSize<u32>,

    pub _num_bytes_screen_buffers: usize,
    pub trail_buffer: wgpu::Buffer,
}

// Data regenerated each frame
pub struct Frame {
    pub output: wgpu::SurfaceTexture,
    pub push_constants: shared::ShaderConstants,
}

// We create buffers in bulk, because many shaders share them
pub fn create_buffers(program_init: &ProgramInit<'_>) -> ProgramBuffers {
    let size = program_init.window.inner_size();
    println!("Width and height {}, {}", size.width, size.height);
    let empty_bytes = bytes_from_trail_map_size(size);
    let num_bytes = empty_bytes.len();

    let trail_buffer = program_init.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Trail buffer"),
        contents: &empty_bytes,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC,
    });

    let buffers = ProgramBuffers {
        screen_size: size,
        _num_bytes_screen_buffers: num_bytes,
        trail_buffer,
    };
    buffers
}

pub fn create_program_frame(program_buffers: &ProgramBuffers, output: wgpu::SurfaceTexture, configuration: &ConfigurationValues, start: &Instant, last_time: &mut Instant) -> Frame {
    let time = start.elapsed().as_secs_f32();
    let delta_time = last_time.elapsed().as_secs_f32();
    if delta_time < configuration.globals.delta_time {
        thread::sleep(Duration::from_secs_f32(configuration.globals.delta_time - delta_time));
    }
    *last_time = std::time::Instant::now();
    let push_constants = ShaderConstants {
        width: program_buffers.screen_size.width,
        height: program_buffers.screen_size.height,
        time,
        delta_time: configuration.globals.delta_time * configuration.globals.time_scale,
    };
    let frame = Frame {
        output,
        push_constants,
    };
    frame
}

pub fn bytes_from_trail_map_size(size: PhysicalSize<u32>) -> Vec<u8> {
    let alignment = wgpu::COPY_BUFFER_ALIGNMENT as u32;
    let num_pixels = ((size.width * size.height).div_ceil(alignment) * alignment) as usize;
    let empty_bytes = std::iter::repeat(0 as u32)
        .take(num_pixels)
        .flat_map(u32::to_ne_bytes)
        .collect::<Vec<_>>();
    empty_bytes
}

pub trait Slot {
    // Data that is created during init
    type Init;
    // Data that is regenerated when the window resizes
    type Buffers;
    fn create(program_init: &ProgramInit<'_>, program_buffers: &ProgramBuffers, configuration: &ConfigurationValues) -> Self;
    fn create_buffers(program_init: &ProgramInit<'_>, program_buffers: &ProgramBuffers, init: &Self::Init) -> Self::Buffers;
    fn recreate_buffers(&mut self, program_init: &ProgramInit<'_>, program_buffers: &ProgramBuffers);
    fn on_loop(&mut self, program_init: &ProgramInit<'_>, program_buffers: &ProgramBuffers, program_frame: &Frame, configuration: &mut ConfigurationValues);
}
