use std::time::{Duration, Instant};
use winit::event::{ElementState, MouseButton, WindowEvent};
use crate::configuration::ConfigurationValues;
use shared::{ClickMode, MouseConstants};
use crate::program::*;
use glam::{uvec2, Vec2, vec2};

const ENTRY_POINT: &str = "mouse_cs";

pub struct SlotMouse {
    pub init: SlotMouseInit,
    pub buffers: SlotMouseBuffers,
    pub mouse_click: Option<ClickStart>,
    pub mouse_position: Vec2,
}

pub struct SlotMouseInit {
    pub pipeline: wgpu::ComputePipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

pub struct SlotMouseBuffers {
    pub bind_group: wgpu::BindGroup,
}

pub struct ClickStart {
    pub _pos: Vec2,
    pub time: Instant,
}

impl Slot for SlotMouse {
    type Init = SlotMouseInit;
    type Buffers = SlotMouseBuffers;

    fn create(program_init: &ProgramInit<'_>, program_buffers: &ProgramBuffers, _configuration: &ConfigurationValues) -> Self {
        let bind_group_layout = program_init.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    count: None,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        has_dynamic_offset: false,
                        min_binding_size: None,
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                    },
                },
            ],
        });

        let pipeline_layout = program_init.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::COMPUTE,
                range: 0..std::mem::size_of::<MouseConstants>() as u32,
            }],
        });

        // Compute
        let pipeline = program_init.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            compilation_options: Default::default(),
            cache: None,
            label: None,
            layout: Some(&pipeline_layout),
            module: &program_init.module,
            entry_point: Some(ENTRY_POINT),
        });

        let init = SlotMouseInit {
            pipeline,
            bind_group_layout,
        };
        let buffers = Self::create_buffers(program_init, program_buffers, &init);
        Self {
            init,
            buffers,
            mouse_click: None,
            mouse_position: vec2(0.0, 0.0),
        }
    }

    fn create_buffers(program_init: &ProgramInit<'_>, program_buffers: &ProgramBuffers, init: &Self::Init) -> Self::Buffers {
        let bind_group = program_init.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute bind group"),
            layout: &init.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: program_buffers.trail_buffer.as_entire_binding(),
                }
            ],
        });
        SlotMouseBuffers {
            bind_group,
        }
    }

    fn recreate_buffers(&mut self, program_init: &ProgramInit<'_>, program_buffers: &ProgramBuffers) {
        let buffers = Self::create_buffers(program_init, program_buffers, &self.init);
        self.buffers = buffers;
    }

    fn on_loop(&mut self, program_init: &ProgramInit<'_>, program_buffers: &ProgramBuffers, _program_frame: &Frame<'_>, configuration: &mut ConfigurationValues) {
        let screen_size = program_init.window.inner_size();
        let mouse_constants = MouseConstants {
            screen_size: uvec2(screen_size.width, screen_size.height),
            map_size: uvec2(program_buffers.map_size.x, program_buffers.map_size.y),
            click_mode: configuration.globals.click_mode.encode(),
            mouse_down: self.mouse_click.is_some() as u32,
            mouse_position: self.mouse_position,
            last_mouse_position: Default::default(),
            brush_size: configuration.globals.brush_size,
            _padding: 0.0,
        };
        // Run compute pass
        let mut compute_encoder =
            program_init.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut cpass = compute_encoder.begin_compute_pass(&Default::default());
            cpass.set_bind_group(0, &self.buffers.bind_group, &[]);
            cpass.set_pipeline(&self.init.pipeline);
            cpass.set_push_constants(
                0,
                bytemuck::bytes_of(&mouse_constants),
            );
            cpass.dispatch_workgroups(program_buffers.map_size.x.div_ceil(8), program_buffers.map_size.y.div_ceil(8), 1);
        }
        program_init.queue.submit([compute_encoder.finish()]);
    }
}

impl SlotMouse {
    pub fn handle_input(&mut self, configuration: &mut ConfigurationValues, event: &WindowEvent) {
        match event {
            WindowEvent::CursorMoved { device_id: _, position } => {
                self.mouse_position = vec2(position.x as f32, position.y as f32);
            }
            WindowEvent::CursorEntered { device_id: _ } => {}
            WindowEvent::CursorLeft { device_id: _ } => {}
            WindowEvent::MouseInput { device_id: _, state, button } => {
                println!("Button press {:?} {:?}", button, state);
                if let MouseButton::Left = button {
                    match state {
                        ElementState::Pressed => {
                            let click_start = ClickStart {
                                _pos: self.mouse_position,
                                time: Instant::now(),
                            };
                            self.mouse_click = Some(click_start);
                        }
                        ElementState::Released => {
                            if let Some(start) = self.mouse_click.take() {
                                if start.time.elapsed() > Duration::from_secs_f32(3.0) {
                                    if !configuration.show_menu {
                                        configuration.show_menu = true;
                                    }
                                } else {
                                    match configuration.globals.click_mode {
                                        ClickMode::Disabled => {}
                                        ClickMode::ShowMenu => {
                                            configuration.show_menu = true;
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
