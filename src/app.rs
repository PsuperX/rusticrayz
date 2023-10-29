use crate::{layer::Layer, raytracer::Raytracer};
use tracing::info;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

const MSAA_SAMPLES: u32 = 1;

pub struct Application {
    window: Window,
    event_loop: Option<EventLoop<()>>,
    wgpu_ctx: WgpuCtx,

    egui_state: egui_winit::State,
    egui_ctx: egui::Context,
    egui_renderer: egui_wgpu::Renderer,

    layers: Vec<Box<dyn Layer>>,

    is_running: bool,
    needs_resize: bool,
}

impl Application {
    pub fn new(title: impl Into<String>) -> Self {
        // Setup window
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title(title)
            .build(&event_loop)
            .expect("Create window");

        // Setup wgpu
        let wgpu_ctx = WgpuCtx::new(&window);

        // Setup egui
        let egui_state = egui_winit::State::new(&window);
        // TODO:
        // egui_state.init_accesskit();
        let egui_ctx = egui::Context::default();
        let egui_renderer = egui_wgpu::renderer::Renderer::new(
            &wgpu_ctx.device,
            wgpu_ctx.format,
            None,
            MSAA_SAMPLES,
        );

        Self {
            window,
            event_loop: Some(event_loop),
            wgpu_ctx,
            egui_state,
            egui_ctx,
            egui_renderer,
            layers: vec![],
            is_running: false,
            needs_resize: false,
        }
    }

    pub fn run(&mut self) {
        self.is_running = true;

        // TODO: find a better place for this
        let raytracer = Raytracer::new(&mut self.wgpu_ctx);
        self.push_layer(Box::new(raytracer));

        use winit::platform::run_return::EventLoopExtRunReturn;
        self.event_loop
            .take()
            .unwrap()
            .run_return(move |event, _, control_flow| {
                // You should change this if you want to render continuosly
                *control_flow = ControlFlow::Wait;

                match event {
                    Event::WindowEvent { event, .. } => {
                        self.handle_window_event(event, control_flow)
                    }
                    Event::RedrawRequested(_) => {
                        if self.needs_resize {
                            self.wgpu_ctx.handle_resize(&self.window.inner_size());
                            self.needs_resize = false;
                        }

                        self.render();
                    }
                    _ => {}
                }
            });
    }

    pub fn close(mut self) {
        for layer in self.layers.iter_mut() {
            layer.on_detach();
        }
    }

    pub fn push_layer(&mut self, mut layer: Box<dyn Layer>) {
        layer.on_attach(&mut self.wgpu_ctx);
        self.layers.push(layer);
    }

    fn handle_window_event(&mut self, event: WindowEvent, control_flow: &mut ControlFlow) {
        // Map window event to egui event
        let response = self.egui_state.on_event(&self.egui_ctx, &event);
        if response.repaint {
            self.window.request_redraw();
        }
        if response.consumed {
            return;
        }

        match event {
            // TODO:
            // WindowEvent::CursorMoved { position, .. } => {
            //     cursor_position = Some(position);
            // }
            // WindowEvent::ModifiersChanged(new_modifiers) => {
            //     modifiers = new_modifiers;
            // }
            WindowEvent::Resized(_) => {
                self.needs_resize = true;
            }
            WindowEvent::CloseRequested => {
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    }

    fn render(&mut self) {
        match self.wgpu_ctx.surface.get_current_texture() {
            Ok(frame) => {
                // let mut encoder = self
                //     .wgpu_ctx
                //     .device
                //     .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                //
                // let egui_raw_input = self.egui_state.take_egui_input(&self.window);
                // let egui_full_output = self.egui_ctx.run(egui_raw_input, |egui_ctx| {
                //     for layer in self.layers.iter_mut() {
                //         layer.on_ui_render(egui_ctx);
                //     }
                // });
                // self.egui_state.handle_platform_output(
                //     &self.window,
                //     &self.egui_ctx,
                //     egui_full_output.platform_output,
                // );
                // let egui_primitives = self.egui_ctx.tessellate(egui_full_output.shapes);
                // let size = self.window.inner_size();
                // let screen_descriptor = egui_wgpu::renderer::ScreenDescriptor {
                //     size_in_pixels: [size.width, size.height],
                //     pixels_per_point: self.egui_state.pixels_per_point(),
                // };
                // for (id, image_delta) in egui_full_output.textures_delta.set {
                //     self.egui_renderer.update_texture(
                //         &self.wgpu_ctx.device,
                //         &self.wgpu_ctx.queue,
                //         id,
                //         &image_delta,
                //     );
                // }
                // self.egui_renderer.update_buffers(
                //     &self.wgpu_ctx.device,
                //     &self.wgpu_ctx.queue,
                //     &mut encoder,
                //     &egui_primitives,
                //     &screen_descriptor,
                // );

                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let mut commands = vec![];
                // Draw my stuff
                for layer in self.layers.iter_mut() {
                    commands.push(layer.on_draw_frame(&self.wgpu_ctx, &view));
                }

                // {
                //     let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                //         label: Some("Render Pass"),
                //         // where to draw to
                //         color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                //             view: &view,
                //             resolve_target: None,
                //             ops: wgpu::Operations {
                //                 load: wgpu::LoadOp::Load,
                //                 store: true,
                //             },
                //         })],
                //         depth_stencil_attachment: None, // TODO: do i need this?
                //     });
                //
                //     // Draw egui
                //     render_pass.push_debug_group("egui");
                //     self.egui_renderer.render(
                //         &mut render_pass,
                //         &egui_primitives,
                //         &screen_descriptor,
                //     );
                // }
                // commands.push(encoder.finish());

                // Then we submit the work
                self.wgpu_ctx.queue.submit(commands);
                frame.present();

                // Free unused textures
                // for id in egui_full_output.textures_delta.free {
                //     self.egui_renderer.free_texture(&id);
                // }

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
                    self.window.request_redraw();
                }
            },
        }
    }
}

pub struct Viewport {
    pub width: u32,
    pub height: u32,
}

pub struct WgpuCtx {
    pub instance: wgpu::Instance,
    pub surface: wgpu::Surface,
    pub format: wgpu::TextureFormat,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub viewport: Viewport,
}

impl WgpuCtx {
    fn new(window: &Window) -> Self {
        let default_backend = wgpu::Backends::PRIMARY;
        let backend = wgpu::util::backend_bits_from_env().unwrap_or(default_backend);

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: backend,
            ..Default::default()
        });
        let surface = unsafe { instance.create_surface(window) }.expect("Create surface");

        let (format, (device, queue)) = pollster::block_on(async {
            let adapter =
                wgpu::util::initialize_adapter_from_env_or_default(&instance, Some(&surface))
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

        let mut ret = Self {
            instance,
            surface,
            format,
            device,
            queue,
            viewport: Viewport {
                width: 0,
                height: 0,
            },
        };
        ret.handle_resize(&window.inner_size());
        ret
    }

    pub fn handle_resize(&mut self, physical_size: &winit::dpi::PhysicalSize<u32>) {
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.format,
            width: physical_size.width,
            height: physical_size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };
        self.surface.configure(&self.device, &surface_config);
        self.viewport = Viewport {
            width: physical_size.width,
            height: physical_size.height,
        };
    }
}
