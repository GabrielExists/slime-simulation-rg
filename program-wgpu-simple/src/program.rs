use std::ops::Deref;
use std::thread;
use std::time::{Duration, Instant};
use crate::configuration::create_agent_stats_all;
use glam::{uvec2, UVec2};
use crate::configuration::RESIZE_MAP_WITH_WINDOW;
use crate::configuration::ConfigurationValues;
use rand::Rng;
use crate::configuration::{GLOBALS, TRAIL_STATS};
use wgpu::SurfaceTexture;
use winit::event::{WindowEvent};
use wgpu::util::DeviceExt;
use shared::{ShaderConstants, INTS_PER_PIXEL};
use crate::slot_agents::SlotAgents;
use crate::slot_diffuse::SlotDiffuse;
use crate::slot_mouse::SlotMouse;
use crate::slot_egui::SlotEgui;
use crate::slot_render::SlotRender;

pub struct Program<'window> {
    program_init: ProgramInit<'window>,
    program_buffers: ProgramBuffers,
    slot_agents: SlotAgents,
    slot_diffuse: SlotDiffuse,
    slot_mouse: SlotMouse,
    slot_render: SlotRender,
    slot_egui: SlotEgui,
    configuration: ConfigurationValues,
    first_frame: bool,
}

// Data that is created at program init
pub struct Handles<'window> {
    pub device: &'window wgpu::Device,
    pub surface_format: &'window wgpu::TextureFormat,
    pub module: &'window wgpu::ShaderModule,
    pub queue: &'window wgpu::Queue,
    pub window: &'window winit::window::Window,
}

pub struct ProgramInit<'window> {
    pub handles: Handles<'window>,
    pub trail_stats_buffer: wgpu::Buffer,
}
impl<'window> Deref for ProgramInit<'window> {
    type Target = Handles<'window>;

    fn deref(&self) -> &Self::Target {
        &self.handles
    }
}

// Data regenerated buffer size is changed
pub struct ProgramBuffers {
    pub map_size: UVec2,

    pub num_bytes_screen_buffers: usize,
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
    fn on_loop(&mut self, program_init: &ProgramInit<'_>, program_buffers: &ProgramBuffers, program_frame: &Frame<'_>, configuration: &mut ConfigurationValues);
}

impl Program<'_> {
    pub fn new(handles: Handles<'_>) -> Program<'_> {
        let configuration = ConfigurationValues {
            shader_config_changed: false,
            globals: GLOBALS,
            agent_stats: create_agent_stats_all(),
            trail_stats: TRAIL_STATS,
            scale_factor: 1.0,
            show_menu: true,
            respawn: false,
            reset_trails: false,
            playing: true,
        };

        let trail_stats_bytes = Self::bytes_from_trail_stats(&configuration);
        let trail_stats_buffer = handles.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Trail buffer"),
            contents: &trail_stats_bytes,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        });
        let program_init = ProgramInit {
            handles,
            trail_stats_buffer,
        };

        let program_buffers = Self::create_buffers(&program_init, &configuration);
        let slot_agents = SlotAgents::create(&program_init, &program_buffers, &configuration);
        let slot_diffuse = SlotDiffuse::create(&program_init, &program_buffers, &configuration);
        let slot_mouse = SlotMouse::create(&program_init, &program_buffers, &configuration);
        let slot_render = SlotRender::create(&program_init, &program_buffers, &configuration);
        let slot_egui = SlotEgui::create(&program_init, &program_buffers, &configuration);
        Program {
            program_init,
            program_buffers,
            slot_agents,
            slot_diffuse,
            slot_mouse,
            slot_render,
            slot_egui,
            configuration,
            first_frame: true,
        }
    }

    // We create buffers that are shared across shaders here
    pub fn create_buffers(program_init: &ProgramInit<'_>, configuration: &ConfigurationValues) -> ProgramBuffers {
        let size = if RESIZE_MAP_WITH_WINDOW {
            let size = program_init.window.inner_size();
            uvec2(size.width, size.height)
        } else {
            uvec2(configuration.globals.map_width, configuration.globals.map_height)
        };
        println!("Map width and height {}, {}", size.x, size.y);
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
            map_size: size,
            num_bytes_screen_buffers: num_bytes,
            trail_buffer,
        };
        buffers
    }

    pub(crate) fn recreate_buffers(&mut self) {
        self.program_buffers = Self::create_buffers(&self.program_init, &self.configuration);
        self.slot_agents.recreate_buffers(&self.program_init, &self.program_buffers);
        self.slot_diffuse.recreate_buffers(&self.program_init, &self.program_buffers);
        self.slot_mouse.recreate_buffers(&self.program_init, &self.program_buffers);
        self.slot_render.recreate_buffers(&self.program_init, &self.program_buffers);
        self.slot_egui.recreate_buffers(&self.program_init, &self.program_buffers);
    }

    pub(crate) fn on_loop(&mut self, output: &mut SurfaceTexture, start: &Instant, last_time: &mut Instant) {
        let time = start.elapsed().as_secs_f32();
        let mut delta_time = last_time.elapsed().as_secs_f32();
        if self.first_frame {
            delta_time = 0.0;
            self.first_frame = false;
        } else {
            let fixed_delta_time = self.configuration.globals.fixed_delta_time * rand::rng().random_range(0.9..1.1);
            if delta_time < fixed_delta_time {
                thread::sleep(Duration::from_secs_f32(fixed_delta_time - delta_time));
            }
            delta_time = fixed_delta_time;
        }
        *last_time = std::time::Instant::now();
        let screen_size = self.program_init.window.inner_size();
        let push_constants = ShaderConstants {
            screen_size: uvec2(screen_size.width, screen_size.height),
            map_size: self.program_buffers.map_size,
            time,
            delta_time: delta_time * self.configuration.globals.time_scale,
            padding_1: 0.0,
            padding_2: 0.0,
            background_color: self.configuration.globals.background_color,
        };
        if self.configuration.reset_trails {
            self.configuration.reset_trails = false;
            let bytes = Self::bytes_from_trail_map_size(self.program_buffers.map_size);
            self.program_init.queue.write_buffer(&self.program_buffers.trail_buffer, 0, &bytes);
            self.program_init.queue.submit([]);
            self.first_frame = true;
        }
        // Update buffers
        if self.configuration.shader_config_changed {
            let trail_stats_bytes = Self::bytes_from_trail_stats(&self.configuration);
            self.program_init.queue.write_buffer(&self.program_init.trail_stats_buffer, 0, &trail_stats_bytes);
            self.program_init.queue.submit([]);
        }
        let frame = Frame {
            output,
            push_constants,
        };
        if self.configuration.playing {
            for _ in 0..self.configuration.globals.compute_steps_per_render {
                self.slot_agents.on_loop(&self.program_init, &self.program_buffers, &frame, &mut self.configuration);
                self.slot_diffuse.on_loop(&self.program_init, &self.program_buffers, &frame, &mut self.configuration);
            }
        }
        self.slot_mouse.on_loop(&self.program_init, &self.program_buffers, &frame, &mut self.configuration);
        self.slot_render.on_loop(&self.program_init, &self.program_buffers, &frame, &mut self.configuration);
        self.slot_egui.on_loop(&self.program_init, &self.program_buffers, &frame, &mut self.configuration);
    }

    pub(crate) fn handle_input(&mut self, event: &WindowEvent) {
        let consumed = self.slot_egui.handle_input(&self.program_init.window, &event);
        if !consumed {
            self.slot_mouse.handle_input(&mut self.configuration, &event);
        }
    }

    pub fn bytes_from_trail_map_size(size: UVec2) -> Vec<u8> {
        let alignment = wgpu::COPY_BUFFER_ALIGNMENT as u32;
        let num_pixels = ((size.x * size.y).div_ceil(alignment) * alignment) as usize;
        let empty_bytes = std::iter::repeat(0 as u32)
            .take(num_pixels * INTS_PER_PIXEL as usize)
            .flat_map(u32::to_ne_bytes)
            .collect::<Vec<_>>();
        empty_bytes
    }
    fn bytes_from_trail_stats(configuration: &ConfigurationValues) -> Vec<u8> {
        let trail_stats_bytes = configuration.trail_stats.iter().flat_map(|trail_stats|
            bytemuck::bytes_of(trail_stats).to_vec()
        ).collect::<Vec<_>>();
        trail_stats_bytes
    }
}

