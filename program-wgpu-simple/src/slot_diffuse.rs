use crate::configuration::ConfigurationValues;
use shared::ShaderConstants;
use crate::program::*;
use wgpu::util::DeviceExt;

const HORIZONTAL_ENTRY_POINT: &str = "diffuse_cs";

pub struct SlotDiffuse {
    pub init: SlotDiffuseInit,
    pub buffers: SlotDiffuseBuffers,
}

pub struct SlotDiffuseInit {
    pub pipeline: wgpu::ComputePipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub trail_stats_buffer: wgpu::Buffer,
}

pub struct SlotDiffuseBuffers {
    pub diffuse_input_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl Slot for SlotDiffuse {
    type Init = SlotDiffuseInit;
    type Buffers = SlotDiffuseBuffers;

    fn create(program_init: &ProgramInit<'_>, program_buffers: &ProgramBuffers, configuration: &ConfigurationValues) -> Self {
        let trail_stats_bytes = Self::bytes_from_trail_stats(configuration);
        let trail_stats_buffer = program_init.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Trail buffer"),
            contents: &trail_stats_bytes,
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
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
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
            entry_point: HORIZONTAL_ENTRY_POINT,
        });

        let init = SlotDiffuseInit {
            pipeline,
            bind_group_layout,
            trail_stats_buffer,
        };
        let buffers = Self::create_buffers(program_init, program_buffers, &init);
        Self {
            init,
            buffers,
        }
    }

    fn create_buffers(program_init: &ProgramInit<'_>, program_buffers: &ProgramBuffers, init: &Self::Init) -> Self::Buffers {
        let empty_bytes = Program::bytes_from_trail_map_size(program_init.window.inner_size());
        let diffuse_input_buffer = program_init.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Diffuse intermediate buffer"),
            contents: &empty_bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });
        let bind_group = program_init.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute bind group"),
            layout: &init.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: init.trail_stats_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: diffuse_input_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: program_buffers.trail_buffer.as_entire_binding(),
                }
            ],
        });
        SlotDiffuseBuffers {
            diffuse_input_buffer,
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
            let trail_stats_bytes = Self::bytes_from_trail_stats(configuration);
            program_init.queue.write_buffer(&self.init.trail_stats_buffer, 0, &trail_stats_bytes);
            program_init.queue.submit([]);
        }
        // Run compute pass
        let mut compute_encoder =
            program_init.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        compute_encoder.copy_buffer_to_buffer(&program_buffers.trail_buffer, 0, &self.buffers.diffuse_input_buffer, 0, program_buffers.num_bytes_screen_buffers as u64);

        {
            let mut cpass = compute_encoder.begin_compute_pass(&Default::default());
            cpass.set_bind_group(0, &self.buffers.bind_group, &[]);
            cpass.set_pipeline(&self.init.pipeline);
            cpass.set_push_constants(
                0,
                bytemuck::bytes_of(&program_frame.push_constants),
            );
            cpass.dispatch_workgroups(program_buffers.screen_size.width.div_ceil(8), program_buffers.screen_size.height.div_ceil(8), 1);
        }
        program_init.queue.submit([compute_encoder.finish()]);
    }
}

impl SlotDiffuse {
    fn bytes_from_trail_stats(configuration: &ConfigurationValues) -> Vec<u8> {
        let trail_stats_bytes = configuration.trail_stats.iter().flat_map(|trail_stats|
            bytemuck::bytes_of(trail_stats).to_vec()
        ).collect::<Vec<_>>();
        trail_stats_bytes
    }
}