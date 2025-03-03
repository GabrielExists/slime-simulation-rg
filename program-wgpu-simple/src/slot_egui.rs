use egui::Context;
use egui_wgpu::Renderer;
use egui_winit::State;
use wgpu::StoreOp;
use winit::event::WindowEvent;
use crate::program::*;


pub struct SlotEgui {
    pub menu_values: MenuValues,
    pub state: State,
    pub renderer: Renderer,
}
pub struct MenuValues {
    scale_factor: f32,
}

impl Slot for SlotEgui {
    type Init = ();
    type Buffers = ();

    fn create(program_init: &ProgramInit<'_>, _program_buffers: &ProgramBuffers) -> Self {
        let egui_context = Context::default();

        let egui_state = egui_winit::State::new(
            egui_context,
            egui::viewport::ViewportId::ROOT,
            &program_init.window,
            Some(program_init.window.scale_factor() as f32),
            None,
            Some(2 * 1024), // default dimension is 2048
        );
        let egui_renderer = Renderer::new(
            &program_init.device,
            program_init.surface_format,
            None,
            1,
            true,
        );

        Self {
            menu_values: MenuValues {
                scale_factor: 1.0
            },
            state: egui_state,
            renderer: egui_renderer,
        }
    }

    fn create_buffers(_program_init: &ProgramInit<'_>, _program_buffers: &ProgramBuffers, _init: &Self::Init) -> Self::Buffers {
        ()
    }
    fn recreate_buffers(&mut self, _program_init: &ProgramInit<'_>, _program_buffers: &ProgramBuffers) {
    }

    fn on_loop(&mut self, program_init: &ProgramInit<'_>, program_buffers: &ProgramBuffers, frame: &Frame) {
        {
            let window = &program_init.window;
            let screen_descriptor = egui_wgpu::ScreenDescriptor {
                size_in_pixels: [program_buffers.width, program_buffers.height],
                pixels_per_point: window.scale_factor() as f32
                    * self.menu_values.scale_factor,
            };
            let surface_view = frame.output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            let mut encoder = program_init
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

            let raw_input = self.state.take_egui_input(window);
            self.state.egui_ctx().begin_pass(raw_input);

            egui::Window::new("winit + egui + wgpu says hello!")
                .resizable(true)
                .vscroll(true)
                .default_open(false)
                //     self.state.egui_ctx()
                .show(self.state.egui_ctx(), |ui| {
                    ui.label("Label!");

                    if ui.button("Button!").clicked() {
                        println!("boom!")
                    }

                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label(format!(
                            "Pixels per point: {}",
                            self.state.egui_ctx().pixels_per_point()
                        ));
                        if ui.button("-").clicked() {
                            self.menu_values.scale_factor = (self.menu_values.scale_factor - 0.1).max(0.3);
                        }
                        if ui.button("+").clicked() {
                            self.menu_values.scale_factor = (self.menu_values.scale_factor + 0.1).min(3.0);
                        }
                    });
                });

            self.state.egui_ctx().set_pixels_per_point(screen_descriptor.pixels_per_point);

            let full_output = self.state.egui_ctx().end_pass();

            self.state
                .handle_platform_output(window, full_output.platform_output);

            let tris = self
                .state
                .egui_ctx()
                .tessellate(full_output.shapes, self.state.egui_ctx().pixels_per_point());
            for (id, image_delta) in &full_output.textures_delta.set {
                self.renderer
                    .update_texture(&program_init.device, &program_init.queue, *id, image_delta);
            }
            self.renderer
                .update_buffers(&program_init.device, &program_init.queue, &mut encoder, &tris, &screen_descriptor);
            let rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_view,
                    resolve_target: None,
                    ops: egui_wgpu::wgpu::Operations {
                        load: egui_wgpu::wgpu::LoadOp::Load,
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                label: Some("egui main render pass"),
                occlusion_query_set: None,
            });

            self.renderer
                .render(&mut rpass.forget_lifetime(), &tris, &screen_descriptor);
            for x in &full_output.textures_delta.free {
                self.renderer.free_texture(x)
            }

            program_init.queue.submit(Some(encoder.finish()));
        }
    }
}

impl SlotEgui {
    pub fn handle_input(&mut self, window: &winit::window::Window, event: &WindowEvent) {
        let _ = self.state.on_window_event(window, event);
    }
}
