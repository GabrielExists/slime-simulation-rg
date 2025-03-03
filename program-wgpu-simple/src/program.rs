use std::thread;
use std::time::{Duration, Instant};
use configuration::{DELTA_TIME, TIME_SCALE};
use wgpu::util::DeviceExt;
use crate::configuration;
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
    pub width: u32,
    pub height: u32,

    pub num_bytes_screen_buffers: usize,
    pub trail_buffer: wgpu::Buffer,
    pub pixel_input_buffer: wgpu::Buffer,
}

// Data regenerated each frame
pub struct Frame {
    pub output: wgpu::SurfaceTexture,
    pub push_constants: shared::ShaderConstants,
}

// We create buffers in bulk, because many shaders share them
pub fn create_buffers(program_init: &ProgramInit) -> ProgramBuffers {
    let alignment = wgpu::COPY_BUFFER_ALIGNMENT as u32;
    let width = program_init.window.inner_size().width;
    let height = program_init.window.inner_size().height;
    println!("Width and height {}, {}", width, height);
    let num_pixels = ((width * height).div_ceil(alignment) * alignment) as usize;

    let empty_bytes = std::iter::repeat(0 as u32)
        .take(num_pixels)
        .flat_map(u32::to_ne_bytes)
        .collect::<Vec<_>>();
    let num_bytes = empty_bytes.len();

    let trail_buffer = program_init.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Trail buffer"),
        contents: &empty_bytes,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC,
    });

    let pixel_input_buffer = program_init.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Pixel input buffer"),
        contents: &empty_bytes,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC,
    });

    let buffers = ProgramBuffers {
        width,
        height,
        num_bytes_screen_buffers: num_bytes,
        trail_buffer,
        pixel_input_buffer,
    };
    buffers
}

pub fn create_program_frame(program_buffers: &ProgramBuffers, output: wgpu::SurfaceTexture, start: &Instant, last_time: &mut Instant) -> Frame {
    let time = start.elapsed().as_secs_f32();
    let delta_time = last_time.elapsed().as_secs_f32();
    if delta_time < DELTA_TIME {
        thread::sleep(Duration::from_secs_f32(DELTA_TIME - delta_time));
    }
    *last_time = std::time::Instant::now();
    let push_constants = ShaderConstants {
        width: program_buffers.width,
        height: program_buffers.height,
        time,
        delta_time: DELTA_TIME * TIME_SCALE,
    };
    let frame = Frame {
        output,
        push_constants,
    };
    frame
}

pub trait Slot {
    // Data that is created during init
    type Init;
    // Data that is regenerated when the window resizes
    type Buffers;
    fn create(program_init: &ProgramInit, program_buffers: &ProgramBuffers) -> Self;
    fn create_buffers(program_init: &ProgramInit, program_buffers: &ProgramBuffers, init: &Self::Init) -> Self::Buffers;
    fn recreate_buffers(&mut self, program_init: &ProgramInit, program_buffers: &ProgramBuffers);
    fn on_loop(&mut self, program_init: &ProgramInit, program_buffers: &ProgramBuffers, program_frame: &Frame);
}
