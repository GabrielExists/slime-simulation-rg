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
    pub compute_pipeline: wgpu::ComputePipeline,
    pub compute_bind_group_layout: wgpu::BindGroupLayout,
    pub agents_buffer: wgpu::Buffer,
}
pub struct SlotAgentsBuffers {
    pub compute_bind_group: wgpu::BindGroup,
}

impl Slot for SlotAgents {
    type Init = SlotAgentsInit;
    type Buffers = SlotAgentsBuffers;

    fn create(program_init: &ProgramInit, program_buffers: &ProgramBuffers) -> Self {
        let agent_bytes = std::iter::repeat(())
            .take(configuration::NUM_AGENTS as usize)
            .flat_map(|()| {
                let agent = shared::Agent {
                    x: 100.0,//program_buffers.width as f32 / 2.0,
                    y: 100.0,// program_buffers.height as f32 / 2.0,
                    angle: rand::rng().random_range(0..1000) as f32 / (std::f32::consts::PI * 2.0),
                };
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
        let compute_bind_group_layout = program_init.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let compute_pipeline_layout = program_init.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute pipeline layout"),
            bind_group_layouts: &[&compute_bind_group_layout],
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::COMPUTE,
                range: 0..std::mem::size_of::<ShaderConstants>() as u32,
            }],
        });

        // Compute
        let compute_pipeline = program_init.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            compilation_options: Default::default(),
            cache: None,
            label: None,
            layout: Some(&compute_pipeline_layout),
            module: &program_init.module,
            entry_point: Some(CS_ENTRY_POINT),
        });

        let init = SlotAgentsInit {
            compute_pipeline,
            compute_bind_group_layout,
            agents_buffer,
        };
        let buffers = Self::create_buffers(program_init, program_buffers, &init);
        Self {
            init,
            buffers,
        }
    }

    fn create_buffers(program_init: &ProgramInit, program_buffers: &ProgramBuffers, init: &Self::Init) -> Self::Buffers {
        let compute_bind_group = program_init.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("compute bind group"),
            layout: &init.compute_bind_group_layout,
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
            compute_bind_group,
        }
    }

    fn recreate_buffers(&mut self, program_init: &ProgramInit, program_buffers: &ProgramBuffers) {
        let buffers = Self::create_buffers(program_init, program_buffers, &self.init);
        self.buffers = buffers;
    }

    fn on_loop(&mut self, program_init: &ProgramInit, program_buffers: &ProgramBuffers, program_frame: &Frame) {
        // Run compute pass
        let mut compute_encoder =
            program_init.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut cpass = compute_encoder.begin_compute_pass(&Default::default());
            cpass.set_bind_group(0, &self.buffers.compute_bind_group, &[]);
            cpass.set_pipeline(&self.init.compute_pipeline);
            cpass.set_push_constants(
                0,
                bytemuck::bytes_of(&program_frame.push_constants),
            );
            cpass.dispatch_workgroups(configuration::NUM_AGENTS.div_ceil(16), 1, 1);
        }

        compute_encoder.copy_buffer_to_buffer(
            &program_buffers.trail_buffer,
            0,
            &program_buffers.pixel_input_buffer,
            0,
            (program_buffers.num_bytes_screen_buffers) as wgpu::BufferAddress,
        );
        program_init.queue.submit([compute_encoder.finish()]);
    }
}