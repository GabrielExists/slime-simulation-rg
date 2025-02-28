use configuration::{SPAWN_MODE, SpawnMode};
use rand::Rng;
use shared::ShaderConstants;
use crate::slots::*;
use wgpu::util::DeviceExt;
use crate::configuration;

const CS_ENTRY_POINT: &str = "main_cs";

pub struct SlotAgents {
    pub init: SlotAgentsInit,
    pub buffers: SlotAgentsBuffers,
}

pub struct SlotAgentsInit {
    pub pipeline: wgpu::ComputePipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub agents_buffer: wgpu::Buffer,
}

pub struct SlotAgentsBuffers {
    pub bind_group: wgpu::BindGroup,
}

impl Slot for SlotAgents {
    type Init = SlotAgentsInit;
    type Buffers = SlotAgentsBuffers;

    fn create(program_init: &ProgramInit, program_buffers: &ProgramBuffers) -> Self {
        let agent_bytes = std::iter::repeat(())
            .take(configuration::NUM_AGENTS as usize)
            .flat_map(|()| {
                let agent = spawn_agent(program_buffers, &SPAWN_MODE);
                bytemuck::bytes_of(&agent).to_vec()
            })
            .collect::<Vec<_>>();

        let agents_buffer = program_init.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Agent buffer"),
            contents: &agent_bytes,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        });
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
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
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
                range: 0..std::mem::size_of::<ShaderConstants>() as u32,
            }],
        });

        // Compute
        let pipeline = program_init.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            compilation_options: Default::default(),
            cache: None,
            label: None,
            layout: Some(&pipeline_layout),
            module: &program_init.module,
            entry_point: Some(CS_ENTRY_POINT),
        });

        let init = SlotAgentsInit {
            pipeline,
            bind_group_layout,
            agents_buffer,
        };
        let buffers = Self::create_buffers(program_init, program_buffers, &init);
        Self {
            init,
            buffers,
        }
    }

    fn create_buffers(program_init: &ProgramInit, program_buffers: &ProgramBuffers, init: &Self::Init) -> Self::Buffers {
        let bind_group = program_init.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("compute bind group"),
            layout: &init.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: init.agents_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: program_buffers.trail_buffer.as_entire_binding(),
                },
            ],
        });
        SlotAgentsBuffers {
            bind_group,
        }
    }

    fn recreate_buffers(&mut self, program_init: &ProgramInit, program_buffers: &ProgramBuffers) {
        let buffers = Self::create_buffers(program_init, program_buffers, &self.init);
        self.buffers = buffers;
    }

    fn on_loop(&mut self, program_init: &ProgramInit, program_buffers: &ProgramBuffers, program_frame: &Frame) {
        // Run compute pass
        let mut encoder =
            program_init.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut cpass = encoder.begin_compute_pass(&Default::default());
            cpass.set_bind_group(0, &self.buffers.bind_group, &[]);
            cpass.set_pipeline(&self.init.pipeline);
            cpass.set_push_constants(
                0,
                bytemuck::bytes_of(&program_frame.push_constants),
            );
            cpass.dispatch_workgroups(configuration::NUM_AGENTS.div_ceil(16), 1, 1);
        }

        encoder.copy_buffer_to_buffer(
            &program_buffers.trail_buffer,
            0,
            &program_buffers.pixel_input_buffer,
            0,
            (program_buffers.num_bytes_screen_buffers) as wgpu::BufferAddress,
        );
        program_init.queue.submit([encoder.finish()]);
    }
}

fn spawn_agent(program_buffers: &ProgramBuffers, spawn_mode: &SpawnMode) -> shared::Agent {
    let center_x = program_buffers.width as f32 / 2.0;
    let center_y = program_buffers.height as f32 / 2.0;
    match spawn_mode {
        SpawnMode::CenterFacingOutwards => {
            shared::Agent {
                x: program_buffers.width as f32 / 2.0,
                y: program_buffers.height as f32 / 2.0,
                angle: get_random_angle(),
            }
        }
        SpawnMode::PointFacingOutwards { x, y } => {
            shared::Agent {
                x: *x,
                y: *y,
                angle: get_random_angle(),
            }
        }
        SpawnMode::CircleFacingInwards { max_distance } => {
            let max_number = 100000;
            let random_angle = get_random_angle();
            let random_fraction = rand::rng().random_range(0..max_number) as f32 / max_number as f32;
            let random_distance = random_fraction * *max_distance;
            shared::Agent {
                x: center_x + random_angle.cos() * random_distance,
                y: center_y + random_angle.sin() * random_distance,
                angle: std::f32::consts::PI + random_angle,
            }
        }
        SpawnMode::EvenlyDistributed => {
            shared::Agent {
                x: rand::rng().random_range(0..program_buffers.width * 10) as f32 / 10.0,
                y: rand::rng().random_range(0..program_buffers.height * 10) as f32 / 10.0,
                angle: get_random_angle(),
            }
        }
        SpawnMode::CircumferenceFacingInward { distance } => {
            let random_angle = get_random_angle();
            shared::Agent {
                x: center_x + random_angle.cos() * distance,
                y: center_y + random_angle.sin() * distance,
                angle: std::f32::consts::PI + random_angle,
            }
        },
        SpawnMode::CircumferenceFacingOutward { distance } => {
            let random_angle = get_random_angle();
            shared::Agent {
                x: center_x + random_angle.cos() * distance,
                y: center_y + random_angle.sin() * distance,
                angle: random_angle,
            }
        },
        SpawnMode::CircumferenceFacingRandom { distance } => {
            let random_angle = get_random_angle();
            shared::Agent {
                x: center_x + random_angle.cos() * distance,
                y: center_y + random_angle.sin() * distance,
                angle: get_random_angle(),
            }
        },
        SpawnMode::CircumferenceFacingClockwise { distance } => {
            let random_angle = get_random_angle();
            shared::Agent {
                x: center_x + random_angle.cos() * distance,
                y: center_y + random_angle.sin() * distance,
                angle: std::f32::consts::PI / 2.0 + random_angle,
            }
        },
    }
}

fn get_random_angle() -> f32 {
    let max_number = 100000;
    let random_fraction = rand::rng().random_range(0..max_number) as f32 / max_number as f32;
    let random_angle = random_fraction * (std::f32::consts::PI * 2.0);
    random_angle
}