use rand::Rng;
use wgpu::{BindGroupLayout, COPY_BUFFER_ALIGNMENT, Device};
use shared::{AgentStats, ShaderConstants};
use wgpu::util::DeviceExt;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

struct Buffers {
    pub width: u32,
    pub height: u32,

    pub num_agents: u32,
    pub agents_buffer: wgpu::Buffer,

    pub num_bytes_screen_buffers: usize,
    pub trail_buffer: wgpu::Buffer,
    pub pixel_input_buffer: wgpu::Buffer,

    pub compute_bind_group: wgpu::BindGroup,
    pub render_bind_group: wgpu::BindGroup,
}

fn _print_type_name<T>(_: T) {
    println!("{}", std::any::type_name::<T>());
}

#[allow(clippy::match_wild_err_arm)]
pub fn run() {
    let mut event_loop_builder = EventLoop::with_user_event();
    let event_loop = event_loop_builder.build().unwrap();

    // FIXME(eddyb) incomplete `winit` upgrade, follow the guides in:
    // https://github.com/rust-windowing/winit/releases/tag/v0.30.0
    #[allow(deprecated)]
        let window = event_loop
        .create_window(
            Window::default_attributes()
                .with_title("Rust GPU - wgpu")
                .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0)),
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
        .unwrap_or(wgpu::Backends::VULKAN | wgpu::Backends::METAL);
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

    let (
        compute_pipeline_layout,
        render_pipeline_layout,
        compute_pipeline,
        render_pipeline,
        compute_bind_group_layout,
        render_bind_group_layout,
    ) = create_pipeline(
        &device,
        surface_with_config.as_ref().map_or_else(
            |pending| pending.preferred_format,
            |(_, surface_config)| surface_config.format,
        ),
    );

    let start = std::time::Instant::now();
    let mut last_time = start;


    // Compute
    // let readback_buffer = device.create_buffer(&wgpu::BufferDescriptor {
    //     label: None,
    //     size: num_pixels as wgpu::BufferAddress,
    //     // Can be read to the CPU, and can be copied from the shader's storage buffer
    //     usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
    //     mapped_at_creation: false,
    // });

    let mut buffers = get_buffers(&device, &compute_bind_group_layout, &render_bind_group_layout, window.inner_size());

    // FIXME(eddyb) incomplete `winit` upgrade, follow the guides in:
    // https://github.com/rust-windowing/winit/releases/tag/v0.30.0
    #[allow(deprecated)]
    event_loop.run(|event, event_loop_window_target| {
        // Have the closure take ownership of the resources.
        // `event_loop.run` never returns, therefore we must do this to ensure
        // the resources are properly cleaned up.
        let _ = (&instance, &adapter, &render_pipeline_layout, &compute_pipeline_layout);
        let render_pipeline = &render_pipeline;

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
                event: WindowEvent::Resized(size),
                ..
            } => {
                if size.width != 0 && size.height != 0 {
                    // Recreate the swap chain with the new size
                    if let Ok((surface, surface_config)) = &mut surface_with_config {
                        surface_config.width = size.width;
                        surface_config.height = size.height;
                        surface.configure(&device, surface_config);
                        buffers = get_buffers(&device, &compute_bind_group_layout, &render_bind_group_layout, size);
                    }
                }
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                let time = start.elapsed().as_secs_f32();
                let delta_time = last_time.elapsed().as_secs_f32();
                last_time = std::time::Instant::now();
                let push_constants = ShaderConstants {
                    width: buffers.width,
                    height: buffers.height,
                    time,
                    delta_time,
                    num_agents: buffers.num_agents,
                    agent_stats: [AgentStats {
                        velocity: 0.1,
                    }],
                };

                // Graphics
                window.request_redraw();

                // Run compute pass
                let mut compute_encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                {
                    let mut cpass = compute_encoder.begin_compute_pass(&Default::default());
                    cpass.set_bind_group(0, &buffers.compute_bind_group, &[]);
                    cpass.set_pipeline(&compute_pipeline);
                    cpass.set_push_constants(
                        0,
                        bytemuck::bytes_of(&push_constants),
                    );
                    cpass.dispatch_workgroups(buffers.num_agents.div_ceil(16), 1, 1);
                }

                compute_encoder.copy_buffer_to_buffer(
                    &buffers.trail_buffer,
                    0,
                    &buffers.pixel_input_buffer,
                    0,
                    (buffers.num_bytes_screen_buffers) as wgpu::BufferAddress,
                );
                queue.submit([compute_encoder.finish()]);


                if let Ok((surface, surface_config)) = &mut surface_with_config {
                    let output = match surface.get_current_texture() {
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
                    let output_view = output
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());
                    let mut graphics_encoder = device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                    {
                        let mut rpass: wgpu::RenderPass<'_> = graphics_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

                        rpass.set_pipeline(render_pipeline);
                        rpass.set_bind_group(0, &buffers.render_bind_group, &[]);
                        rpass.set_push_constants(
                            wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                            0,
                            bytemuck::bytes_of(&push_constants),
                        );
                        rpass.draw(0..3, 0..1);
                    }

                    queue.submit([graphics_encoder.finish()]);
                    output.present();

                    // // Compute
                    // let buffer_slice = readback_buffer.slice(..);
                    // buffer_slice.map_async(wgpu::MapMode::Read, |r| r.unwrap());
                    // // NOTE(eddyb) `poll` should return only after the above callbacks fire
                    // // (see also https://github.com/gfx-rs/wgpu/pull/2698 for more details).
                    // device.poll(wgpu::Maintain::Wait);
                    //
                    // let data = buffer_slice.get_mapped_range();
                    // let _result = data
                    //     .chunks_exact(4)
                    //     .map(|b| u32::from_ne_bytes(b.try_into().unwrap()))
                    //     .collect::<Vec<_>>();
                    // drop(data);
                    // readback_buffer.unmap();

                    // let mut max = 0;
                    // for (src, out) in src_range.clone().zip(result.iter().copied()) {
                    //     if out == u32::MAX {
                    //         println!("{src}: overflowed");
                    //         break;
                    //     } else if out > max {
                    //         max = out;
                    //         // Should produce <https://oeis.org/A006877>
                    //         println!("{src}: {out}");
                    //     }
                    // }
                }
            }
            Event::WindowEvent {
                event:
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
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
            } => event_loop_window_target.exit(),
            _ => {}
        }
    }).unwrap();
}

fn get_buffers(device: &Device, compute_bind_group_layout: &BindGroupLayout, render_bind_group_layout: &BindGroupLayout, size: winit::dpi::PhysicalSize<u32>) -> Buffers {
    let alignment = COPY_BUFFER_ALIGNMENT as u32;
    let width = size.width;
    let height = size.height;
    println!("Width and height {}, {}", width, height);
    let num_pixels = ((width * height).div_ceil(alignment) * alignment) as usize;

    let empty_bytes = std::iter::repeat(0 as u32)
        .take(num_pixels)
        .flat_map(u32::to_ne_bytes)
        .collect::<Vec<_>>();
    let num_bytes = empty_bytes.len();

    let num_agents = 256;
    let agent_bytes = std::iter::repeat(())
        .take(num_agents)
        .flat_map(|()| {
            let agent = shared::Agent {
                x: rand::rng().random_range(0..width) as f32,
                y: rand::rng().random_range(0..height) as f32,
                angle: rand::rng().random_range(0..1000) as f32 / (std::f32::consts::PI * 2.0),
            };
            bytemuck::bytes_of(&agent).to_vec()
        })
        .collect::<Vec<_>>();

    let agents_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Agent buffer"),
        contents: &agent_bytes,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC,
    });
    let trail_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Trail buffer"),
        contents: &empty_bytes,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC,
    });

    let pixel_input_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Pixel input buffer"),
        contents: &empty_bytes,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC,
    });

    let compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("compute bind group"),
        layout: &compute_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: agents_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: trail_buffer.as_entire_binding(),
            },
        ],
    });

    let render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("render bind group"),
        layout: &render_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: pixel_input_buffer.as_entire_binding(),
            },
        ],
    });
    let buffers = Buffers {
        width,
        height,
        num_agents: num_agents as u32,
        agents_buffer,
        num_bytes_screen_buffers: num_bytes,
        trail_buffer,
        pixel_input_buffer,
        compute_bind_group,
        render_bind_group,
    };
    buffers
}

fn create_pipeline(
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
) -> (wgpu::PipelineLayout, wgpu::PipelineLayout, wgpu::ComputePipeline, wgpu::RenderPipeline, wgpu::BindGroupLayout, wgpu::BindGroupLayout) {
    let create_module = |module| {
        let wgpu::ShaderModuleDescriptorSpirV { label, source } = module;
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label,
            source: wgpu::ShaderSource::SpirV(source),
        })
    };

    // Merged
    let vs_entry_point = "main_vs";
    let fs_entry_point = "main_fs";
    let cs_entry_point = "main_cs";
    let module_raw = wgpu::include_spirv_raw!(env!("shader_slime.spv"));
    let module = &create_module(module_raw);

    // Graphics
    let render_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

    // Compute
    let compute_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

    // Merged
    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render pipeline layout"),
        bind_group_layouts: &[&render_bind_group_layout],
        push_constant_ranges: &[wgpu::PushConstantRange {
            stages: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            range: 0..std::mem::size_of::<ShaderConstants>() as u32,
        }],
    });
    let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Compute pipeline layout"),
        bind_group_layouts: &[&compute_bind_group_layout],
        push_constant_ranges: &[wgpu::PushConstantRange {
            stages: wgpu::ShaderStages::COMPUTE,
            range: 0..std::mem::size_of::<ShaderConstants>() as u32,
        }],
    });


    // Graphics
    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        cache: None,
        label: Some("Render pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module,
            entry_point: Some(vs_entry_point),
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
            module,
            entry_point: Some(fs_entry_point),
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview: None,
    });


    // Compute
    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        compilation_options: Default::default(),
        cache: None,
        label: None,
        layout: Some(&compute_pipeline_layout),
        module,
        entry_point: Some(cs_entry_point),
    });

    (compute_pipeline_layout, render_pipeline_layout, compute_pipeline, render_pipeline, compute_bind_group_layout, render_bind_group_layout)
}

// Graphics
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
