use egui::menu::MenuState;
use winit::event::WindowEvent;
use crate::egui_tools::EguiRenderer;
use shared::ShaderConstants;
use crate::program::*;

const VS_ENTRY_POINT: &'static str = "main_vs";
const FS_ENTRY_POINT: &'static str = "main_fs";


pub struct SlotEgui {
    pub init: SlotEguiInit,
    pub buffers: SlotEguiBuffers,
    pub menu_values: MenuValues,
}

pub struct SlotEguiInit {
    // pub pipeline: wgpu::RenderPipeline,
    // pub bind_group_layout: wgpu::BindGroupLayout,
    pub egui_renderer: EguiRenderer,
}

pub struct SlotEguiBuffers {
    // pub bind_group: wgpu::BindGroup,
}

pub struct MenuValues {
    scale_factor: f32,
}

impl Slot for SlotEgui {
    type Init = SlotEguiInit;
    type Buffers = SlotEguiBuffers;

    fn create(program_init: &ProgramInit, program_buffers: &ProgramBuffers) -> Self {
        let egui_renderer = EguiRenderer::new(
            &program_init.device,
            program_init.surface_format,
            None,
            1, &program_init.window);

        let init = SlotEguiInit {
            // pipeline,
            // bind_group_layout,
            egui_renderer,
        };
        let window_data = Self::create_buffers(&program_init, program_buffers, &init);
        Self {
            init,
            buffers: window_data,
            menu_values: MenuValues {
                scale_factor: 1.0
            },
        }
    }

    fn create_buffers(program_init: &ProgramInit, program_buffers: &ProgramBuffers, init: &Self::Init) -> Self::Buffers {
        SlotEguiBuffers {
        }
    }
    fn recreate_buffers(&mut self, program_init: &ProgramInit, program_buffers: &ProgramBuffers) {
        let buffers = Self::create_buffers(program_init, program_buffers, &self.init);
        self.buffers = buffers;
    }

    fn on_loop(&mut self, program_init: &ProgramInit, program_buffers: &ProgramBuffers, frame: &Frame) {
        {
            let screen_descriptor = egui_wgpu::ScreenDescriptor {
                size_in_pixels: [program_buffers.width, program_buffers.height],
                pixels_per_point: program_init.window.scale_factor() as f32
                    * self.menu_values.scale_factor,
            };
            let surface_view = frame.output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            let mut encoder = program_init
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

            // let window = self.window.as_ref().unwrap();
            self.init.egui_renderer.begin_frame(&program_init.window);

            egui::Window::new("winit + egui + wgpu says hello!")
                .resizable(true)
                .vscroll(true)
                .default_open(false)
                .show(self.init.egui_renderer.context(), |ui| {
                    ui.label("Label!");

                    if ui.button("Button!").clicked() {
                        println!("boom!")
                    }

                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label(format!(
                            "Pixels per point: {}",
                            self.init.egui_renderer.context().pixels_per_point()
                        ));
                        if ui.button("-").clicked() {
                            self.menu_values.scale_factor = (self.menu_values.scale_factor - 0.1).max(0.3);
                        }
                        if ui.button("+").clicked() {
                            self.menu_values.scale_factor = (self.menu_values.scale_factor + 0.1).min(3.0);
                        }
                    });
                });

            self.init.egui_renderer.end_frame_and_draw(
                &program_init.device,
                &program_init.queue,
                &mut encoder,
                &program_init.window,
                &surface_view,
                screen_descriptor,
            );
            program_init.queue.submit(Some(encoder.finish()));
        }
        // let output_view = frame.output
        //     .texture
        //     .create_view(&wgpu::TextureViewDescriptor::default());
        // let mut encoder = program_init.device
        //     .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        // {
        //     let mut rpass: wgpu::RenderPass<'_> = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        //         label: None,
        //         color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        //             view: &output_view,
        //             resolve_target: None,
        //             ops: wgpu::Operations {
        //                 load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
        //                 store: wgpu::StoreOp::Store,
        //             },
        //         })],
        //         depth_stencil_attachment: None,
        //         ..Default::default()
        //     });
        //
        //     rpass.set_pipeline(&self.init.pipeline);
        //     rpass.set_bind_group(0, &self.buffers.bind_group, &[]);
        //     rpass.set_push_constants(
        //         wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
        //         0,
        //         bytemuck::bytes_of(&frame.push_constants),
        //     );
        //     rpass.draw(0..3, 0..1);
        // }
        //
        // program_init.queue.submit([encoder.finish()]);
    }
}

impl SlotEgui {
    pub fn handle_input(&mut self, window: &winit::window::Window, event: &WindowEvent) {
        self.init.egui_renderer.handle_input(window, event);
    }
}
