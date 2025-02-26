use std::borrow::Cow;
use std::time::Duration;
use shared::ShaderConstants;
use wgpu::util::DeviceExt;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

mod shaders {
    // include!(concat!(env!("OUT_DIR"), "/entry_points.rs"));
    #[allow(non_upper_case_globals)]
    pub const main_fs: &str = "main_fs";
    #[allow(non_upper_case_globals)]
    pub const main_vs: &str = "main_vs";
    #[allow(non_upper_case_globals)]
    pub const main_cs: &str = "main_cs";
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
            winit::window::Window::default_attributes()
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

    // Compute
    // Timestamping may not be supported
    let timestamping = adapter.features().contains(wgpu::Features::TIMESTAMP_QUERY)
        && adapter
        .features()
        .contains(wgpu::Features::TIMESTAMP_QUERY_INSIDE_PASSES);


    // Merged
    let required_features = wgpu::Features::PUSH_CONSTANTS | if timestamping {
        wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TIMESTAMP_QUERY_INSIDE_PASSES
    } else {
        wgpu::Features::empty()
    };
    // Compute
    if !timestamping {
        eprintln!(
            "Adapter reports that timestamping is not supported - no timing information will be available"
        );
    }

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
                required_features,
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

    // Load the shaders from disk


    let (
        render_pipeline_layout,
        compute_pipeline_layout,
        mut render_pipeline,
        compute_pipeline,
        compute_bind_group_layout,
    ) = create_pipeline(
        &device,
        surface_with_config.as_ref().map_or_else(
            |pending| pending.preferred_format,
            |(_, surface_config)| surface_config.format,
        ),
    );

    let start = std::time::Instant::now();

    // FIXME(eddyb) incomplete `winit` upgrade, follow the guides in:
    // https://github.com/rust-windowing/winit/releases/tag/v0.30.0
    #[allow(deprecated)]
    event_loop.run(|event, event_loop_window_target| {
        // Have the closure take ownership of the resources.
        // `event_loop.run` never returns, therefore we must do this to ensure
        // the resources are properly cleaned up.
        let _ = (&instance, &adapter, &render_pipeline_layout, &compute_pipeline_layout);
        let render_pipeline = &mut render_pipeline;

        event_loop_window_target.set_control_flow(ControlFlow::Wait);
        match event {
            Event::Resumed => {
                // Avoid holding onto to multiple surfaces at the same time
                // (as it's undetected and can confusingly break e.g. Wayland).
                //
                // FIXME(eddyb) create the window and `wgpu::Surface` on either
                // `Event::NewEvents(StartCause::Init)`, or `Event::Resumed`,
                // which is becoming recommended on (almost) all platforms, see:
                // - https://github.com/rust-windowing/winit/releases/tag/v0.30.0
                // - https://github.com/gfx-rs/wgpu/blob/v23/examples/src/framework.rs#L139-L161
                //   (note wasm being handled differently due to its `<canvas>`)
                if let Ok((_, surface_config)) = &surface_with_config {
                    // HACK(eddyb) can't move out of `surface_with_config` as
                    // it's a closure capture, and also the `Err(_)` variant
                    // has a payload so that needs to be filled with something.
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
                    }
                }
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                // Graphics
                window.request_redraw();

                // Compute
                let timestamp_period: Option<f32> = if timestamping {
                    Some(queue.get_timestamp_period())
                } else {
                    None
                };

                // Compute
                let top = 2u32.pow(20);
                let src_range = 1..top;

                let src = src_range
                    .clone()
                    .flat_map(u32::to_ne_bytes)
                    .collect::<Vec<_>>();

                // Compute
                let readback_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: None,
                    size: src.len() as wgpu::BufferAddress,
                    // Can be read to the CPU, and can be copied from the shader's storage buffer
                    usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });

                let storage_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Collatz Conjecture Input"),
                    contents: &src,
                    usage: wgpu::BufferUsages::STORAGE
                        | wgpu::BufferUsages::COPY_DST
                        | wgpu::BufferUsages::COPY_SRC,
                });

                let (timestamp_buffer, timestamp_readback_buffer) = if timestamping {
                    let timestamp_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("Timestamps buffer"),
                        size: 16,
                        usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
                        mapped_at_creation: false,
                    });

                    let timestamp_readback_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                        label: None,
                        size: 16,
                        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: true,
                    });
                    timestamp_readback_buffer.unmap();

                    (Some(timestamp_buffer), Some(timestamp_readback_buffer))
                } else {
                    (None, None)
                };

                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &compute_bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: storage_buffer.as_entire_binding(),
                    }],
                });

                let queries = if timestamping {
                    Some(device.create_query_set(&wgpu::QuerySetDescriptor {
                        label: None,
                        count: 2,
                        ty: wgpu::QueryType::Timestamp,
                    }))
                } else {
                    None
                };

                // Run compute pass
                let mut compute_encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                {
                    let mut cpass = compute_encoder.begin_compute_pass(&Default::default());
                    cpass.set_bind_group(0, &bind_group, &[]);
                    cpass.set_pipeline(&compute_pipeline);
                    if timestamping {
                        if let Some(queries) = queries.as_ref() {
                            cpass.write_timestamp(queries, 0);
                        }
                    }
                    cpass.dispatch_workgroups(src_range.len() as u32 / 64, 1, 1);
                    if timestamping {
                        if let Some(queries) = queries.as_ref() {
                            cpass.write_timestamp(queries, 1);
                        }
                    }
                }

                compute_encoder.copy_buffer_to_buffer(
                    &storage_buffer,
                    0,
                    &readback_buffer,
                    0,
                    src.len() as wgpu::BufferAddress,
                );

                if timestamping {
                    if let (Some(queries), Some(timestamp_buffer), Some(timestamp_readback_buffer)) = (
                        queries.as_ref(),
                        timestamp_buffer.as_ref(),
                        timestamp_readback_buffer.as_ref(),
                    ) {
                        compute_encoder.resolve_query_set(queries, 0..2, timestamp_buffer, 0);
                        compute_encoder.copy_buffer_to_buffer(
                            timestamp_buffer,
                            0,
                            timestamp_readback_buffer,
                            0,
                            timestamp_buffer.size(),
                        );
                    }
                }

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

                        let time = start.elapsed().as_secs_f32();

                        let push_constants = ShaderConstants {
                            width: window.inner_size().width,
                            height: window.inner_size().height,
                            time,
                        };

                        rpass.set_pipeline(render_pipeline);
                        rpass.set_push_constants(
                            wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                            0,
                            bytemuck::bytes_of(&push_constants),
                        );
                        rpass.draw(0..3, 0..1);
                    }

                    queue.submit([compute_encoder.finish(), graphics_encoder.finish()]);
                    output.present();

                    // Compute
                    let buffer_slice = readback_buffer.slice(..);
                    if timestamping {
                        if let Some(timestamp_readback_buffer) = timestamp_readback_buffer.as_ref() {
                            let timestamp_slice = timestamp_readback_buffer.slice(..);
                            timestamp_slice.map_async(wgpu::MapMode::Read, |r| r.unwrap());
                        }
                    }
                    buffer_slice.map_async(wgpu::MapMode::Read, |r| r.unwrap());
                    // NOTE(eddyb) `poll` should return only after the above callbacks fire
                    // (see also https://github.com/gfx-rs/wgpu/pull/2698 for more details).
                    device.poll(wgpu::Maintain::Wait);

                    if timestamping {
                        if let (Some(timestamp_readback_buffer), Some(timestamp_period)) =
                            (timestamp_readback_buffer.as_ref(), timestamp_period)
                        {
                            {
                                let timing_data = timestamp_readback_buffer.slice(..).get_mapped_range();
                                let timings = timing_data
                                    .chunks_exact(8)
                                    .map(|b| u64::from_ne_bytes(b.try_into().unwrap()))
                                    .collect::<Vec<_>>();

                                println!(
                                    "Took: {:?}",
                                    Duration::from_nanos(
                                        ((timings[1] - timings[0]) as f64 * f64::from(timestamp_period)) as u64
                                    )
                                );
                                drop(timing_data);
                                timestamp_readback_buffer.unmap();
                            }
                        }
                    }

                    let data = buffer_slice.get_mapped_range();
                    let result = data
                        .chunks_exact(4)
                        .map(|b| u32::from_ne_bytes(b.try_into().unwrap()))
                        .collect::<Vec<_>>();
                    drop(data);
                    readback_buffer.unmap();

                    let mut max = 0;
                    for (src, out) in src_range.zip(result.iter().copied()) {
                        if out == u32::MAX {
                            println!("{src}: overflowed");
                            break;
                        } else if out > max {
                            max = out;
                            // Should produce <https://oeis.org/A006877>
                            println!("{src}: {out}");
                        }
                    }
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

fn create_pipeline(
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
) -> (wgpu::PipelineLayout, wgpu::PipelineLayout, wgpu::RenderPipeline, wgpu::ComputePipeline, wgpu::BindGroupLayout) {
    let create_module = |module| {
        let wgpu::ShaderModuleDescriptorSpirV { label, source } = module;
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label,
            source: wgpu::ShaderSource::SpirV(source),
        })
    };

    // Merged
    let vs_entry_point = shaders::main_vs;
    let fs_entry_point = shaders::main_fs;
    let cs_entry_point = shaders::main_cs;
    let module_raw = wgpu::include_spirv_raw!(env!("shader_slime.spv"));
    let module = &create_module(module_raw);

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
        ],
    });

    // Merged
    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render pipeline layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[wgpu::PushConstantRange {
            stages: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            range: 0..std::mem::size_of::<ShaderConstants>() as u32,
        }],
    });
    let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Compute pipeline layout"),
        bind_group_layouts: &[&compute_bind_group_layout],
        push_constant_ranges: &[],
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

    (render_pipeline_layout, compute_pipeline_layout, render_pipeline, compute_pipeline, compute_bind_group_layout)
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
