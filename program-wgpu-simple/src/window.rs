use crate::{
    configuration::RESIZE_MAP_WITH_WINDOW,
    program::Handles,
    program::Program,
};
#[cfg(not(target_arch = "aarch64"))]
use crate::configuration::{DEFAULT_HEIGHT, DEFAULT_WIDTH};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
#[cfg(target_arch = "aarch64")]
use winit::window::Fullscreen;

fn _print_type_name<T>(_: T) {
    println!("{}", std::any::type_name::<T>());
}

#[allow(clippy::match_wild_err_arm)]
pub fn run() {
    let mut event_loop_builder = EventLoop::with_user_event();
    let event_loop = event_loop_builder.build().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    // let mut app = crate::egui_app::App::new();

    // event_loop.run_app(&mut app).expect("Failed to run app");

    // FIXME(eddyb) incomplete `winit` upgrade, follow the guides in:
    // https://github.com/rust-windowing/winit/releases/tag/v0.30.0
    #[allow(deprecated)]
        #[cfg(target_arch = "aarch64")]
        let window = event_loop
        .create_window(
            Window::default_attributes()
                .with_title("Rust GPU - wgpu")
                .with_inner_size(winit::dpi::LogicalSize::new(800.0, 480.0))
                .with_fullscreen(Some(Fullscreen::Borderless(None))),
        )
        .unwrap();
    #[allow(deprecated)]
        #[cfg(not(target_arch = "aarch64"))]
        let window = event_loop
        .create_window(
            Window::default_attributes()
                .with_title("Rust GPU - wgpu")
                .with_inner_size(winit::dpi::LogicalSize::new(DEFAULT_WIDTH as f32, DEFAULT_HEIGHT as f32))
        )
        .unwrap();

    futures::executor::block_on(run_inner(
        event_loop,
        window,
    ));
}

async fn run_inner(
    event_loop: EventLoop<()>,
    window: Window,
) {
    // Common in compute and graphics
    let backends = wgpu::util::backend_bits_from_env()
        .unwrap_or(wgpu::Backends::VULKAN | wgpu::Backends::METAL | wgpu::Backends::GL);
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends,
        dx12_shader_compiler: wgpu::util::dx12_shader_compiler_from_env().unwrap_or_default(),
        ..Default::default()
    });

    // Graphics
    // HACK(eddyb) marker error type for lazily-created surfaces (e.g. on Android).
    struct SurfaceCreationPending {
        preferred_format: wgpu::TextureFormat,
    }

    // Graphics
    // Wait for Resumed event on Android; the surface is only otherwise needed
    // early to find an adapter that can render to this surface.
    let initial_surface = if cfg!(target_os = "android") {
        Err(SurfaceCreationPending {
            preferred_format: wgpu::TextureFormat::Rgba8UnormSrgb,
        })
    } else {
        Ok(instance
            .create_surface(&window)
            .expect("Failed to create surface from window"))
    };

    // Common in compute and graphics
    let adapter = wgpu::util::initialize_adapter_from_env_or_default(
        &instance,
        // Request an adapter which can render to our surface
        initial_surface.as_ref().ok(),
    )
        .await
        .expect("Failed to find an appropriate adapter");

    // Graphics
    let required_limits = wgpu::Limits {
        max_push_constant_size: 128,
        ..Default::default()
    };

    // Common
    // Create the logical device and command queue
    let (device, queue): (wgpu::Device, wgpu::Queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::PUSH_CONSTANTS | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
                required_limits,
                memory_hints: wgpu::MemoryHints::Performance,
            },
            None,
        )
        .await
        .expect("Failed to create device");

    // Graphics
    let mut surface_with_config = initial_surface
        .map(|surface| auto_configure_surface(&adapter, &device, surface, window.inner_size()));

    let surface_format: wgpu::TextureFormat = surface_with_config.as_ref().map_or_else(
        |pending| pending.preferred_format,
        |(_, surface_config)| surface_config.format,
    );
    println!("Preferred surface format: {:?}, actual surface format {:?}", wgpu::TextureFormat::Rgba8UnormSrgb, surface_format);
    fn auto_configure_surface<'a>(
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        surface: wgpu::Surface<'a>,
        size: winit::dpi::PhysicalSize<u32>,
    ) -> (wgpu::Surface<'a>, wgpu::SurfaceConfiguration) {
        let mut surface_config = surface
            .get_default_config(adapter, size.width, size.height)
            .unwrap_or_else(|| {
                panic!(
                    "Missing formats/present modes in surface capabilities: {:#?}",
                    surface.get_capabilities(adapter)
                )
            });

        // FIXME(eddyb) should this be toggled by a CLI arg?
        // NOTE(eddyb) VSync was disabled in the past, but without VSync,
        // especially for simpler shaders, you can easily hit thousands
        // of frames per second, stressing GPUs for no reason.
        surface_config.present_mode = wgpu::PresentMode::AutoVsync;

        surface.configure(device, &surface_config);

        (surface, surface_config)
    }

    // Load shader
    let create_module = |module| {
        let wgpu::ShaderModuleDescriptorSpirV { label, source } = module;
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label,
            source: wgpu::ShaderSource::SpirV(source),
        })
    };
    let module_raw = wgpu::include_spirv_raw!(env!("shader_slime.spv"));
    let module = create_module(module_raw);

    let handles = Handles {
        device: &device,
        surface_format: &surface_format,
        module: &module,
        queue: &queue,
        window: &window,
    };
    let mut program = Program::new(handles);

    let start = std::time::Instant::now();
    let mut last_time = start;

    // FIXME(eddyb) incomplete `winit` upgrade, follow the guides in:
    // https://github.com/rust-windowing/winit/releases/tag/v0.30.0
    #[allow(deprecated)]
    event_loop.run(|event, event_loop_window_target| {
        // Have the closure take ownership of the resources.
        // `event_loop.run` never returns, therefore we must do this to ensure
        // the resources are properly cleaned up.
        let _ = (&instance, &adapter);

        event_loop_window_target.set_control_flow(ControlFlow::Wait);
        match event {
            Event::Resumed => {
                // Avoid holding onto to multiple surfaces at the same time
                // (as it's undetected and can confusingly break e.g. Wayland).
                if let Ok((_, surface_config)) = &surface_with_config {
                    let filler = Err(SurfaceCreationPending {
                        preferred_format: surface_config.format,
                    });
                    drop(std::mem::replace(&mut surface_with_config, filler));
                }

                let new_surface = instance.create_surface(&window)
                    .expect("Failed to create surface from window (after resume)");
                surface_with_config = Ok(auto_configure_surface(
                    &adapter,
                    &device,
                    new_surface,
                    window.inner_size(),
                ));
            }
            Event::Suspended => {
                if let Ok((_, surface_config)) = &surface_with_config {
                    surface_with_config = Err(SurfaceCreationPending {
                        preferred_format: surface_config.format,
                    });
                }
            }
            Event::WindowEvent {
                event: event @ WindowEvent::Resized(size),
                ..
            } => {
                program.handle_input(&event);
                if size.width != 0 && size.height != 0 {
                    // Recreate the swap chain with the new size
                    if let Ok((surface, surface_config)) = &mut surface_with_config {
                        surface_config.width = size.width;
                        surface_config.height = size.height;
                        surface.configure(&device, surface_config);
                        if RESIZE_MAP_WITH_WINDOW {
                            program.recreate_buffers();
                        }
                        last_time = std::time::Instant::now();
                    }
                }
            }
            Event::WindowEvent {
                event: event @ WindowEvent::RedrawRequested,
                ..
            } => {
                program.handle_input(&event);
                window.request_redraw();
                if let Ok((surface, surface_config)) = &mut surface_with_config {
                    let mut output = match surface.get_current_texture() {
                        Ok(surface) => surface,
                        Err(err) => {
                            eprintln!("get_current_texture error: {err:?}");
                            match err {
                                wgpu::SurfaceError::Lost => {
                                    surface.configure(&device, surface_config);
                                }
                                wgpu::SurfaceError::OutOfMemory => {
                                    event_loop_window_target.exit();
                                }
                                _ => (),
                            }
                            return;
                        }
                    };
                    let continue_running = program.on_loop(&mut output, &start, &mut last_time);
                    if !continue_running {
                        event_loop_window_target.exit();
                    }

                    output.present();
                }
            }
            Event::WindowEvent {
                event: event @ WindowEvent::CloseRequested |
                event @ WindowEvent::KeyboardInput {
                    event:
                    winit::event::KeyEvent {
                        logical_key:
                        winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape),
                        state: winit::event::ElementState::Pressed,
                        ..
                    },
                    ..
                },
                ..
            } => {
                program.handle_input(&event);
                event_loop_window_target.exit()
            }
            Event::WindowEvent {
                event, ..
            } => {
                program.handle_input(&event);
            }
            _ => {}
        }
    }).unwrap();

    // */
}

