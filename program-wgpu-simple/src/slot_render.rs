use shared::ShaderConstants;
use crate::program::*;

const VS_ENTRY_POINT: &'static str = "main_vs";
const FS_ENTRY_POINT: &'static str = "main_fs";


pub struct SlotRender {
    pub init: SlotRenderInit,
    pub buffers: SlotRenderBuffers,
}

pub struct SlotRenderInit {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

pub struct SlotRenderBuffers {
    pub bind_group: wgpu::BindGroup,
}

impl Slot for SlotRender {
    type Init = SlotRenderInit;
    type Buffers = SlotRenderBuffers;

    fn create(program_init: &ProgramInit, program_buffers: &ProgramBuffers) -> Self {
        // Graphics
        let bind_group_layout = program_init.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    count: None,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        has_dynamic_offset: false,
                        min_binding_size: None,
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                    },
                },
            ],
        });
        // Merged
        let pipeline_layout = program_init.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                range: 0..std::mem::size_of::<ShaderConstants>() as u32,
            }],
        });
        // Graphics
        let pipeline = program_init.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            cache: None,
            label: Some("Render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &program_init.module,
                entry_point: VS_ENTRY_POINT,
                buffers: &[],
                compilation_options: Default::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                compilation_options: Default::default(),
                module: &program_init.module,
                entry_point: FS_ENTRY_POINT,
                targets: &[Some(wgpu::ColorTargetState {
                    format: program_init.surface_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });
        let init = SlotRenderInit {
            pipeline,
            bind_group_layout,
        };
        let window_data = Self::create_buffers(&program_init, program_buffers, &init);
        Self {
            init,
            buffers: window_data,
        }
    }

    fn create_buffers(program_init: &ProgramInit, program_buffers: &ProgramBuffers, init: &Self::Init) -> Self::Buffers {
        let bind_group = program_init.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("render bind group"),
            layout: &init.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: program_buffers.pixel_input_buffer.as_entire_binding(),
                },
            ],
        });
        SlotRenderBuffers {
            bind_group,
        }
    }
    fn recreate_buffers(&mut self, program_init: &ProgramInit, program_buffers: &ProgramBuffers) {
        let buffers = Self::create_buffers(program_init, program_buffers, &self.init);
        self.buffers = buffers;
    }

    fn on_loop(&mut self, program_init: &ProgramInit, _program_buffers: &ProgramBuffers, frame: &Frame) {
        let output_view = frame.output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = program_init.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass: wgpu::RenderPass<'_> = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });

            rpass.set_pipeline(&self.init.pipeline);
            rpass.set_bind_group(0, &self.buffers.bind_group, &[]);
            rpass.set_push_constants(
                wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                0,
                bytemuck::bytes_of(&frame.push_constants),
            );
            rpass.draw(0..3, 0..1);
        }

        program_init.queue.submit([encoder.finish()]);
    }
}
