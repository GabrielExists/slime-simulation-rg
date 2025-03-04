use std::thread;
use std::time::{Duration, Instant};
use crate::configuration::{AGENT_STATS, GLOBALS, TRAIL_STATS};
use wgpu::SurfaceTexture;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::window::Window;
use crate::configuration_menu::ConfigurationValues;
use wgpu::util::DeviceExt;
use shared::ShaderConstants;
use crate::slot_agents::SlotAgents;
use crate::slot_diffuse::SlotDiffuse;
use crate::slot_egui::SlotEgui;
use crate::slot_render::SlotRender;

pub struct Program<'window> {
    program_init: ProgramInit<'window>,
    program_buffers: ProgramBuffers,
    slot_agents: SlotAgents,
    slot_diffuse: SlotDiffuse,
    slot_render: SlotRender,
    slot_egui: SlotEgui,
    configuration: ConfigurationValues,
}

// Data that is created at program init
pub struct ProgramInit<'window> {
    pub device: &'window wgpu::Device,
    pub surface_format: &'window wgpu::TextureFormat,
    pub module: &'window wgpu::ShaderModule,
    pub queue: &'window wgpu::Queue,
    pub window: &'window winit::window::Window,
}

// Data regenerated when window is resized
pub struct ProgramBuffers {
    pub screen_size: PhysicalSize<u32>,

    pub _num_bytes_screen_buffers: usize,
    pub trail_buffer: wgpu::Buffer,
}

// Data regenerated each frame
pub struct Frame<'output> {
    pub output: &'output mut wgpu::SurfaceTexture,
    pub push_constants: shared::ShaderConstants,
}

/// A slot is the backend that a shader "slots" into.
/// Each shader has individual bindings and render steps that need to be matched in the cpu,
/// and by making the collection a trait we can collect them easily iterate over them
/// or disable one
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

impl Program<'_> {
    pub fn new(program_init: ProgramInit) -> Program {
        let mut configuration = ConfigurationValues {
            globals: GLOBALS,
            agent_stats: AGENT_STATS,
            trail_stats: TRAIL_STATS,
            scale_factor: 1.0,
            show_menu: true,
            respawn: false,
        };
        let mut program_buffers = Self::create_buffers(&program_init);
        let slot_agents = SlotAgents::create(&program_init, &program_buffers, &configuration);
        let slot_diffuse = SlotDiffuse::create(&program_init, &program_buffers, &configuration);
        let slot_render = SlotRender::create(&program_init, &program_buffers, &configuration);
        let slot_egui = SlotEgui::create(&program_init, &program_buffers, &configuration);
        Program {
            program_init,
            program_buffers,
            slot_agents,
            slot_diffuse,
            slot_render,
            slot_egui,
            configuration,
        }
    }

    // We create buffers that are shared across shaders here
    pub fn create_buffers(program_init: &ProgramInit<'_>) -> ProgramBuffers {
        let size = program_init.window.inner_size();
        println!("Width and height {}, {}", size.width, size.height);
        let empty_bytes = Self::bytes_from_trail_map_size(size);
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

    pub(crate) fn recreate_buffers(&mut self) {
        self.program_buffers = Self::create_buffers(&self.program_init);
        self.slot_agents.recreate_buffers(&self.program_init, &self.program_buffers);
        self.slot_diffuse.recreate_buffers(&self.program_init, &self.program_buffers);
        self.slot_render.recreate_buffers(&self.program_init, &self.program_buffers);
        self.slot_egui.recreate_buffers(&self.program_init, &self.program_buffers);
    }

    pub(crate) fn on_loop(&mut self, output: &mut SurfaceTexture, start: &Instant, last_time: &mut Instant) {
        let time = start.elapsed().as_secs_f32();
        let delta_time = last_time.elapsed().as_secs_f32();
        if delta_time < self.configuration.globals.delta_time {
            thread::sleep(Duration::from_secs_f32(self.configuration.globals.delta_time - delta_time));
        }
        *last_time = std::time::Instant::now();
        let push_constants = ShaderConstants {
            width: self.program_buffers.screen_size.width,
            height: self.program_buffers.screen_size.height,
            time,
            delta_time: self.configuration.globals.delta_time * self.configuration.globals.time_scale,
        };
        let frame = Frame {
            output,
            push_constants,
        };
        for _ in 0..self.configuration.globals.compute_steps_per_render {
            self.slot_agents.on_loop(&self.program_init, &self.program_buffers, &frame, &mut self.configuration);
            self.slot_diffuse.on_loop(&self.program_init, &self.program_buffers, &frame, &mut self.configuration);
        }
        self.slot_render.on_loop(&self.program_init, &self.program_buffers, &frame, &mut self.configuration);
        self.slot_egui.on_loop(&self.program_init, &self.program_buffers, &frame, &mut self.configuration);
    }

    pub(crate) fn handle_input(&mut self, event: &WindowEvent) {
        self.slot_egui.handle_input(&self.program_init.window, &event);
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
}

