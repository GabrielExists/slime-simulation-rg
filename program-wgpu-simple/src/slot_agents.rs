use crate::configuration::AgentStatsAll;
use winit::dpi::PhysicalSize;
use crate::configuration::ConfigurationValues;
use rand::Rng;
use shared::{ShaderConstants, SpawnBox, SpawnMode};
use crate::program::*;
use wgpu::util::DeviceExt;

const CS_ENTRY_POINT: &str = "main_cs";

pub struct SlotAgents {
    pub init: SlotAgentsInit,
    pub buffers: SlotAgentsBuffers,
}

pub struct SlotAgentsInit {
    pub pipeline: wgpu::ComputePipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub agents_buffer: wgpu::Buffer,
    pub agent_stats_buffer: wgpu::Buffer,
    pub num_agents: usize,
}

pub struct SlotAgentsBuffers {
    pub bind_group: wgpu::BindGroup,
}

impl Slot for SlotAgents {
    type Init = SlotAgentsInit;
    type Buffers = SlotAgentsBuffers;

    fn create(program_init: &ProgramInit<'_>, program_buffers: &ProgramBuffers, configuration: &ConfigurationValues) -> Self {
        let mut num_agents = 0;
        let agent_bytes = Self::bytes_from_agents(configuration, program_buffers.screen_size, &mut num_agents);

        let agent_stats_bytes = Self::bytes_from_agent_stats(configuration);
        let agent_stats_buffer = program_init.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Agent stats buffer"),
            contents: &agent_stats_bytes,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        });

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
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                    },
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
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
            entry_point: CS_ENTRY_POINT,
        });

        let init = SlotAgentsInit {
            pipeline,
            bind_group_layout,
            agents_buffer,
            agent_stats_buffer,
            num_agents,
        };
        let buffers = Self::create_buffers(program_init, program_buffers, &init);
        Self {
            init,
            buffers,
        }
    }

    fn create_buffers(program_init: &ProgramInit<'_>, program_buffers: &ProgramBuffers, init: &Self::Init) -> Self::Buffers {
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
                    resource: init.agent_stats_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: program_buffers.trail_buffer.as_entire_binding(),
                },
            ],
        });
        SlotAgentsBuffers {
            bind_group,
        }
    }

    fn recreate_buffers(&mut self, program_init: &ProgramInit<'_>, program_buffers: &ProgramBuffers) {
        let buffers = Self::create_buffers(program_init, program_buffers, &self.init);
        self.buffers = buffers;
    }

    fn on_loop(&mut self, program_init: &ProgramInit<'_>, program_buffers: &ProgramBuffers, program_frame: &Frame<'_>, configuration: &mut ConfigurationValues) {
        // Update buffers
        if configuration.shader_config_changed {
            let agent_stats_bytes = Self::bytes_from_agent_stats(configuration);
            program_init.queue.write_buffer(&self.init.agent_stats_buffer, 0, &agent_stats_bytes);
        }
        if configuration.respawn {
            configuration.respawn = false;
            let mut num_agents = 0;
            let agent_bytes = Self::bytes_from_agents(configuration, program_buffers.screen_size, &mut num_agents);
            let agent_buffer = program_init.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Agent buffer recreated"),
                contents: &agent_bytes,
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
            });
            self.init.agents_buffer = agent_buffer;
            self.init.num_agents = num_agents;
            self.recreate_buffers(program_init, program_buffers);
        }

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
            cpass.dispatch_workgroups(self.init.num_agents.div_ceil(256) as u32, 1, 1);
        }

        program_init.queue.submit([encoder.finish()]);
    }
}

impl SlotAgents {
    fn bytes_from_agent_stats(configuration: &ConfigurationValues) -> Vec<u8> {
        let agent_stats_bytes = configuration.agent_stats.iter().flat_map(|stats_all|
            bytemuck::bytes_of(&stats_all.shader_stats).to_vec()
        ).collect::<Vec<_>>();
        agent_stats_bytes
    }

    fn bytes_from_agents(configuration: &ConfigurationValues, size: PhysicalSize<u32>, num_agents: &mut usize) -> Vec<u8> {
        let agent_bytes = configuration.agent_stats
            .iter()
            .enumerate()
            .flat_map(|(channel_index, agent_stats): (usize, &AgentStatsAll)| {
                *num_agents += agent_stats.num_agents;
                std::iter::repeat(())
                    .take(agent_stats.num_agents)
                    .flat_map(move |()| {
                        let agent = spawn_agent(size, &agent_stats.spawn_mode, channel_index as u32);
                        bytemuck::bytes_of(&agent).to_vec()
                    })
            }).collect::<Vec<_>>();
        agent_bytes
    }
}

fn spawn_agent(size: PhysicalSize<u32>, spawn_mode: &SpawnMode, channel_index: u32) -> shared::Agent {
    let center_x = size.width as f32 / 2.0;
    let center_y = size.height as f32 / 2.0;
    match spawn_mode {
        SpawnMode::CenterFacingOutward => {
            shared::Agent {
                x: size.width as f32 / 2.0,
                y: size.height as f32 / 2.0,
                angle: get_random_angle(),
                channel_index,
            }
        }
        SpawnMode::PointFacingOutward { x, y } => {
            shared::Agent {
                x: *x as f32,
                y: *y as f32,
                angle: get_random_angle(),
                channel_index,
            }
        }
        SpawnMode::CircleFacingInward { max_distance } => {
            let max_number = 100000;
            let random_angle = get_random_angle();
            let random_fraction = rand::rng().random_range(0..max_number) as f32 / max_number as f32;
            let random_distance = random_fraction * *max_distance as f32;
            shared::Agent {
                x: center_x + random_angle.cos() * random_distance,
                y: center_y + random_angle.sin() * random_distance,
                angle: std::f32::consts::PI + random_angle,
                channel_index,
            }
        }
        SpawnMode::EvenlyDistributed => {
            shared::Agent {
                x: rand::rng().random_range(0..size.width * 10) as f32 / 10.0,
                y: rand::rng().random_range(0..size.height * 10) as f32 / 10.0,
                angle: get_random_angle(),
                channel_index,
            }
        }
        SpawnMode::CircumferenceFacingInward { distance } => {
            let random_angle = get_random_angle();
            shared::Agent {
                x: center_x + random_angle.cos() * *distance as f32,
                y: center_y + random_angle.sin() * *distance as f32,
                angle: std::f32::consts::PI + random_angle,
                channel_index,
            }
        }
        SpawnMode::CircumferenceFacingOutward { distance } => {
            let random_angle = get_random_angle();
            shared::Agent {
                x: center_x + random_angle.cos() * *distance as f32,
                y: center_y + random_angle.sin() * *distance as f32,
                angle: random_angle,
                channel_index,
            }
        }
        SpawnMode::CircumferenceFacingRandom { distance } => {
            let random_angle = get_random_angle();
            shared::Agent {
                x: center_x + random_angle.cos() * *distance as f32,
                y: center_y + random_angle.sin() * *distance as f32,
                angle: get_random_angle(),
                channel_index,
            }
        }
        SpawnMode::CircumferenceFacingClockwise { distance } => {
            let random_angle = get_random_angle();
            shared::Agent {
                x: center_x + random_angle.cos() * *distance as f32,
                y: center_y + random_angle.sin() * *distance as f32,
                angle: std::f32::consts::PI / 2.0 + random_angle,
                channel_index,
            }
        }
        SpawnMode::BoxFacingRandom { spawn_box: SpawnBox { left, top, box_width, box_height } } => {
            shared::Agent {
                x: rand::rng().random_range(*left as f32..*left as f32 + *box_width as f32),
                y: rand::rng().random_range(*top as f32..*top as f32 + *box_height as f32),
                angle: get_random_angle(),
                channel_index,
            }
        }
    }
}

fn get_random_angle() -> f32 {
    rand::rng().random_range(0.0..std::f32::consts::PI * 2.0)
}