use tracing::info;
use winit::{
    event::{Event, ModifiersState, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

const MSAA_SAMPLES: u32 = 1;
const BACKGROUND_COLOR: wgpu::Color = wgpu::Color::BLACK;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Initialize winit
    let event_loop = EventLoop::new();

    let window = winit::window::Window::new(&event_loop)?;

    let physical_size = window.inner_size();
    // let mut viewport = Viewport::with_physical_size(
    //     Size::new(physical_size.width, physical_size.height),
    //     window.scale_factor(),
    // );
    let mut cursor_position = None;
    let mut modifiers = ModifiersState::default();

    // Initialize wgpu
    let default_backend = wgpu::Backends::PRIMARY;

    let backend = wgpu::util::backend_bits_from_env().unwrap_or(default_backend);

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: backend,
        ..Default::default()
    });
    let surface = unsafe { instance.create_surface(&window) }?;

    let (format, (device, queue)) = pollster::block_on(async {
        let adapter = wgpu::util::initialize_adapter_from_env_or_default(&instance, Some(&surface))
            .await
            .expect("Create adapter");

        let adapter_features = adapter.features();

        let needed_limits = wgpu::Limits::default();

        let capabilities = surface.get_capabilities(&adapter);

        (
            capabilities
                .formats
                .iter()
                .copied()
                .find(wgpu::TextureFormat::is_srgb)
                .or_else(|| capabilities.formats.first().copied())
                .expect("Get preferred format"),
            adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label: None,
                        features: adapter_features & wgpu::Features::default(),
                        limits: needed_limits,
                    },
                    None,
                )
                .await
                .expect("Request device"),
        )
    });

    let surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        width: physical_size.width,
        height: physical_size.height,
        present_mode: wgpu::PresentMode::AutoVsync,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        view_formats: vec![],
    };
    surface.configure(&device, &surface_config);

    let mut needs_resize = false;

    // Initialize egui
    let mut egui_state = egui_winit::State::new(&window);
    // TODO:
    // egui_state.init_accesskit();
    let mut egui_ctx = egui::Context::default();

    let mut egui_renderer = egui_wgpu::renderer::Renderer::new(&device, format, None, MSAA_SAMPLES);

    // Run event loop
    event_loop.run(move |event, _, control_flow| {
        // You should change this if you want to render continuosly
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent { event, .. } => {
                // Map window event to egui event
                let response = egui_state.on_event(&egui_ctx, &event);
                if response.repaint {
                    window.request_redraw();
                }
                if response.consumed {
                    return;
                }

                match event {
                    WindowEvent::CursorMoved { position, .. } => {
                        cursor_position = Some(position);
                    }
                    WindowEvent::ModifiersChanged(new_modifiers) => {
                        modifiers = new_modifiers;
                    }
                    WindowEvent::Resized(_) => {
                        needs_resize = true;
                    }
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => {}
                }
            }
            Event::MainEventsCleared => {
                // If there are events pending
                // if !state.is_queue_empty() {
                //     // We update iced
                //     let _ = state.update(
                //         viewport.logical_size(),
                //         cursor_position
                //             .map(|p| conversion::cursor_position(p, viewport.scale_factor()))
                //             .map(mouse::Cursor::Available)
                //             .unwrap_or(mouse::Cursor::Unavailable),
                //         &mut renderer,
                //         &Theme::Dark,
                //         &renderer::Style {
                //             text_color: Color::WHITE,
                //         },
                //         &mut clipboard,
                //         &mut debug,
                //     );
                //
                //     // and request a redraw
                //     window.request_redraw();
                // }
            }
            Event::RedrawRequested(_) => {
                if needs_resize {
                    let size = window.inner_size();

                    // viewport = Viewport::with_physical_size(
                    //     Size::new(size.width, size.height),
                    //     window.scale_factor(),
                    // );

                    surface.configure(
                        &device,
                        &wgpu::SurfaceConfiguration {
                            format,
                            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                            width: size.width,
                            height: size.height,
                            present_mode: wgpu::PresentMode::AutoVsync,
                            alpha_mode: wgpu::CompositeAlphaMode::Auto,
                            view_formats: vec![],
                        },
                    );

                    needs_resize = false;
                }

                match surface.get_current_texture() {
                    Ok(frame) => {
                        let mut encoder =
                            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                label: None,
                            });

                        let egui_raw_input = egui_state.take_egui_input(&window);
                        let egui_full_output = egui_ctx.run(egui_raw_input, |egui_ctx| {
                            egui::CentralPanel::default().show(egui_ctx, |ui| {
                                ui.label("Hello world!");
                                if ui.button("Click me").clicked() {
                                    info!("Click! :D");
                                }
                            });
                        });
                        egui_state.handle_platform_output(
                            &window,
                            &egui_ctx,
                            egui_full_output.platform_output,
                        );
                        let egui_primitives = egui_ctx.tessellate(egui_full_output.shapes);
                        let screen_descriptor = egui_wgpu::renderer::ScreenDescriptor {
                            size_in_pixels: [window.inner_size().width, window.inner_size().height],
                            pixels_per_point: egui_state.pixels_per_point(),
                        };
                        for (id, image_delta) in egui_full_output.textures_delta.set {
                            egui_renderer.update_texture(&device, &queue, id, &image_delta);
                        }
                        egui_renderer.update_buffers(
                            &device,
                            &queue,
                            &mut encoder,
                            &egui_primitives,
                            &screen_descriptor,
                        );

                        let view = frame
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());

                        {
                            let mut render_pass =
                                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                    label: Some("Render Pass"),
                                    // where to draw to
                                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                        view: &view,
                                        resolve_target: None,
                                        ops: wgpu::Operations {
                                            load: wgpu::LoadOp::Clear(BACKGROUND_COLOR),
                                            store: true,
                                        },
                                    })],
                                    depth_stencil_attachment: None, // TODO: do i need this?
                                });

                            // TODO: Draw my stuff
                            // scene.draw(&mut render_pass);

                            // Draw egui
                            egui_renderer.render(
                                &mut render_pass,
                                &egui_primitives,
                                &screen_descriptor,
                            );
                        }

                        // Then we submit the work
                        queue.submit(Some(encoder.finish()));
                        frame.present();

                        // Free unused textures
                        for id in egui_full_output.textures_delta.free {
                            egui_renderer.free_texture(&id);
                        }

                        // TODO: Update the mouse cursor
                    }
                    Err(error) => match error {
                        wgpu::SurfaceError::OutOfMemory => {
                            panic!(
                                "Swapchain error: {error}. \
                                Rendering cannot continue."
                            )
                        }
                        _ => {
                            // Try rendering again next frame.
                            window.request_redraw();
                        }
                    },
                }
            }
            _ => {}
        }
    })
}
