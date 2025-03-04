use std::thread;
use std::time::{Duration, Instant};
use rand::Rng;
use crate::configuration::{AGENT_STATS, GLOBALS, TRAIL_STATS};
use wgpu::SurfaceTexture;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, MouseButton, WindowEvent};
use crate::configuration_menu::render_configuration_menu;
use crate::configuration_menu::ConfigurationValues;
use wgpu::util::DeviceExt;
use shared::{ClickMode, ClickModeEncoded, ShaderConstants};
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
    mouse_click: Option<ClickStart>,
    mouse_position: (f32, f32),
    first_frame: bool,
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

pub struct ClickStart {
    x: f32,
    y: f32,
    time: Instant,
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
    pub fn new(program_init: ProgramInit<'_>) -> Program<'_> {
        let configuration = ConfigurationValues {
            globals: GLOBALS,
            agent_stats: AGENT_STATS,
            trail_stats: TRAIL_STATS,
            scale_factor: 1.0,
            show_menu: true,
            respawn: false,
            reset_trails: false,
            playing: true,
        };
        let program_buffers = Self::create_buffers(&program_init);
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
            mouse_click: None,
            mouse_position: (0.0, 0.0),
            first_frame: true,
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
        if self.configuration.reset_trails {
            self.configuration.reset_trails = false;
            let bytes = Self::bytes_from_trail_map_size(self.program_buffers.screen_size);
            self.program_init.queue.write_buffer(&self.program_buffers.trail_buffer, 0, &bytes);
            self.program_init.queue.submit([]);
        }
        let time = start.elapsed().as_secs_f32();
        let mut delta_time = last_time.elapsed().as_secs_f32();
        if self.first_frame {
            delta_time = 0.0;
            self.first_frame = false;
        } else {
            let fixed_delta_time = self.configuration.globals.fixed_delta_time * rand::rng().random_range(0.8..1.2);
            if delta_time < fixed_delta_time {
                thread::sleep(Duration::from_secs_f32(fixed_delta_time - delta_time));
                delta_time = fixed_delta_time;
            }
        }
        *last_time = std::time::Instant::now();
        let push_constants = ShaderConstants {
            click_mode: self.configuration.globals.click_mode.encode(),
            width: self.program_buffers.screen_size.width,
            height: self.program_buffers.screen_size.height,
            time,
            delta_time: delta_time * self.configuration.globals.time_scale,
        };
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
        self.slot_render.on_loop(&self.program_init, &self.program_buffers, &frame, &mut self.configuration);
        self.slot_egui.on_loop(&self.program_init, &self.program_buffers, &frame, &mut self.configuration);
    }

    pub(crate) fn handle_input(&mut self, event: &WindowEvent) {
        let consumed = self.slot_egui.handle_input(&self.program_init.window, &event);
        if !consumed {
            match event {
                WindowEvent::CursorMoved { device_id, position } => {
                    self.mouse_position = (position.x as f32, position.y as f32);
                }
                WindowEvent::CursorEntered { device_id } => {
                }
                WindowEvent::CursorLeft { device_id } => {
                }
                WindowEvent::MouseInput{ device_id, state, button }  => {
                    println!("Button press {:?} {:?}", button, state);
                    if let MouseButton::Left = button{
                        match state {
                            ElementState::Pressed => {
                                let click_start = ClickStart {
                                    x: self.mouse_position.0,
                                    y: self.mouse_position.1,
                                    time: Instant::now(),
                                };
                                self.mouse_click = Some(click_start);
                            }
                            ElementState::Released => {
                                if let Some(start) = self.mouse_click.take() {
                                    if start.time.elapsed() > Duration::from_secs_f32(3.0) {
                                        self.configuration.show_menu = !self.configuration.show_menu;
                                    } else {
                                        match self.configuration.globals.click_mode {
                                            ClickMode::Disabled => {}
                                            ClickMode::ShowMenu => {
                                                self.configuration.show_menu = true;
                                            }
                                            ClickMode::PaintTrail(_) => {}
                                            ClickMode::ResetTrail(_) => {}
                                            ClickMode::ResetAllTrails => {}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
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

