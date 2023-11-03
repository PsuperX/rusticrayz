use crate::{app::WgpuCtx, camera::CameraUniform, layer::Layer, scene::Scene, triangle::Triangle};
use egui::{containers::Frame, Margin};
use glam::{vec2, vec3, Vec2, Vec3};
use std::{borrow::Cow, mem};
use tracing::info;

const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
const MAX_TRIANGLE_COUNT: u64 = 32;

pub struct Raytracer {
    // Assets
    scene_data_buffer: wgpu::Buffer,
    objects_buffer: wgpu::Buffer,

    max_bounces: i32,
    samples_per_pixel: i32,
    render_scale: f32,

    // Pipeline stuff
    rt_bind_group_layout: wgpu::BindGroupLayout,
    rt_pipeline: wgpu::ComputePipeline,
    rt_bind_group: wgpu::BindGroup,
    screen_bind_group_layout: wgpu::BindGroupLayout,
    screen_pipeline: wgpu::RenderPipeline,
    screen_bind_group: wgpu::BindGroup,
    sampler: wgpu::Sampler,
}

impl Raytracer {
    pub fn new(ctx: &mut WgpuCtx, max_bounces: i32, samples_per_pixel: i32, render_scale: f32) -> Self {
        let (color_buffer, sampler, scene_data_buffer, objects_buffer) =
            create_assets(ctx, render_scale);
        let (
            rt_bind_group_layout,
            rt_pipeline,
            rt_bind_group,
            screen_bind_group_layout,
            screen_pipeline,
            screen_bind_group,
        ) = create_pipeline(
            ctx,
            &color_buffer,
            &sampler,
            &scene_data_buffer,
            &objects_buffer,
        );

        Self {
            scene_data_buffer,
            objects_buffer,
            max_bounces,
            samples_per_pixel,
            render_scale,
            rt_bind_group_layout,
            rt_pipeline,
            rt_bind_group,
            screen_bind_group_layout,
            screen_pipeline,
            screen_bind_group,
            sampler,
        }
    }
}

impl Layer for Raytracer {
    fn on_ui_render(&mut self, ctx: &egui::Context) {
        let frame = Frame {
            fill: egui::Color32::TRANSPARENT,
            inner_margin: Margin::same(10.0),
            ..Default::default()
        };
        egui::TopBottomPanel::top("my panel")
            .frame(frame)
            .show(ctx, |ui| {
                ui.label(egui::RichText::new("Hello world!").color(egui::Color32::WHITE));
                if ui.button("Click me").clicked() {
                    info!("Click! :D");
                }
            });

        egui::Window::new("My Window").show(ctx, |ui| {
            ui.label(":D");
        });
    }

    fn on_draw_frame(
        &mut self,
        ctx: &WgpuCtx,
        view: &wgpu::TextureView,
        scene: &Scene,
    ) -> wgpu::CommandBuffer {
        let primitives = scene.get_primitives();
        let camera = scene.get_camera();
        let (pixel00_loc, pixel_delta_u, pixel_delta_v) = viewport_vectors(
            vec2(ctx.viewport.width as f32, ctx.viewport.height as f32) * self.render_scale,
            camera.pos,
        );
        let scene_data = SceneData::new(
            camera.get_uniform(),
            self.max_bounces,
            self.samples_per_pixel,
            primitives.len() as i32,
            pixel00_loc,
            pixel_delta_u,
            pixel_delta_v,
        );
        ctx.queue.write_buffer(
            &self.scene_data_buffer,
            0,
            bytemuck::cast_slice(&[scene_data]),
        );

        ctx.queue
            .write_buffer(&self.objects_buffer, 0, bytemuck::cast_slice(primitives));

        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Raytracer encoder"),
            });

        {
            let mut rt_compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Raytracer compute pass"),
            });

            rt_compute_pass.set_pipeline(&self.rt_pipeline);
            rt_compute_pass.set_bind_group(0, &self.rt_bind_group, &[]);
            rt_compute_pass.dispatch_workgroups(ctx.viewport.width, ctx.viewport.height, 1);
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Raytracer render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.screen_pipeline);
            render_pass.set_bind_group(0, &self.screen_bind_group, &[]);
            render_pass.draw(0..6, 0..1);
        }

        encoder.finish()
    }

    fn on_resize(&mut self, ctx: &mut WgpuCtx) {
        let color_buffer_view = create_color_buffer(ctx, self.render_scale)
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.rt_bind_group = create_rt_bind_group(
            ctx,
            &self.rt_bind_group_layout,
            &color_buffer_view,
            &self.scene_data_buffer,
            &self.objects_buffer,
        );
        self.screen_bind_group = create_screen_bind_group(
            ctx,
            &self.screen_bind_group_layout,
            &self.sampler,
            &color_buffer_view,
        );
    }
}

fn create_color_buffer(ctx: &WgpuCtx, scale: f32) -> wgpu::Texture {
    ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Raytracer color buffer"),
        size: wgpu::Extent3d {
            width: (ctx.viewport.width as f32 * scale) as u32,
            height: (ctx.viewport.height as f32 * scale) as u32,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FORMAT,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
        view_formats: &[],
    })
}

fn create_rt_bind_group(
    ctx: &WgpuCtx,
    rt_bind_group_layout: &wgpu::BindGroupLayout,
    color_buffer_view: &wgpu::TextureView,
    scene_data_buffer: &wgpu::Buffer,
    objects_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Raytracer bind group"),
        layout: rt_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(color_buffer_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: scene_data_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: objects_buffer.as_entire_binding(),
            },
        ],
    })
}

fn create_screen_bind_group(
    ctx: &WgpuCtx,
    screen_bind_group_layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    color_buffer_view: &wgpu::TextureView,
) -> wgpu::BindGroup {
    ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Raytracer screen bind group"),
        layout: screen_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(color_buffer_view),
            },
        ],
    })
}

fn create_assets(
    ctx: &WgpuCtx,
    render_scale: f32,
) -> (wgpu::Texture, wgpu::Sampler, wgpu::Buffer, wgpu::Buffer) {
    let color_buffer = create_color_buffer(ctx, render_scale);

    let sampler = ctx.device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("Raytracer screen sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    let scene_data_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Scene data buffer"),
        size: mem::size_of::<SceneData>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let objects_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Objects data buffer"),
        size: MAX_TRIANGLE_COUNT * mem::size_of::<Triangle>() as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    (color_buffer, sampler, scene_data_buffer, objects_buffer)
}

fn create_pipeline(
    ctx: &WgpuCtx,
    color_buffer: &wgpu::Texture,
    sampler: &wgpu::Sampler,
    scene_data_buffer: &wgpu::Buffer,
    objects_buffer: &wgpu::Buffer,
) -> (
    wgpu::BindGroupLayout,
    wgpu::ComputePipeline,
    wgpu::BindGroup,
    wgpu::BindGroupLayout,
    wgpu::RenderPipeline,
    wgpu::BindGroup,
) {
    let rt_bind_group_layout =
        ctx.device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Raytracer compute bind group layout"),
                entries: &[
                    // Output
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: FORMAT,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    // Scene Data
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // Objects Data
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });
    let color_buffer_view = color_buffer.create_view(&wgpu::TextureViewDescriptor::default());
    let rt_bind_group = create_rt_bind_group(
        ctx,
        &rt_bind_group_layout,
        &color_buffer_view,
        scene_data_buffer,
        objects_buffer,
    );

    let rt_pipeline_layout = ctx
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Raytracer compute pipeline layout"),
            bind_group_layouts: &[&rt_bind_group_layout],
            push_constant_ranges: &[],
        });

    let cs_module = ctx
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Raytracer compute shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders/raytracer.wgsl"))),
        });

    let rt_pipeline = ctx
        .device
        .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Raytracer compute pipeline"),
            layout: Some(&rt_pipeline_layout),
            module: &cs_module,
            entry_point: "main",
        });

    let screen_bind_group_layout =
        ctx.device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Raytracer screen layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
            });
    let screen_bind_group =
        create_screen_bind_group(ctx, &screen_bind_group_layout, sampler, &color_buffer_view);

    let screen_pipeline_layout =
        ctx.device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Raytracer screen pipeline layout"),
                bind_group_layouts: &[&screen_bind_group_layout],
                push_constant_ranges: &[],
            });

    let screen_module = ctx
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Raytracer screen shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders/screen.wgsl"))),
        });

    let screen_pipeline = ctx
        .device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Raytracer screen pipeline"),
            layout: Some(&screen_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &screen_module,
                entry_point: "vs_main",
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &screen_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: ctx.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });
    (
        rt_bind_group_layout,
        rt_pipeline,
        rt_bind_group,
        screen_bind_group_layout,
        screen_pipeline,
        screen_bind_group,
    )
}

fn viewport_vectors(screen_size: Vec2, camera_pos: Vec3) -> (Vec3, Vec3, Vec3) {
    let focal_length = 1.0;
    let viewport_height = 2.0;
    let viewport_width = viewport_height * (screen_size.x / screen_size.y);
    let camera_center = camera_pos;

    // Calculate the vectors across the horizontal and down the vertical viewport edges.
    let viewport_u = vec3(viewport_width, 0.0, 0.0);
    let viewport_v = vec3(0.0, -viewport_height, 0.0);

    // Calculate the horizontal and vertical delta vectors from pixel to pixel.
    let pixel_delta_u = viewport_u / screen_size.x;
    let pixel_delta_v = viewport_v / screen_size.y;

    // Calculate the location of the upper left pixel.
    let viewport_upper_left =
        camera_center - vec3(0.0, 0.0, focal_length) - viewport_u / 2.0 - viewport_v / 2.0;
    let pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

    (pixel00_loc, pixel_delta_u, pixel_delta_v)
}

#[repr(C, align(16))]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct SceneData {
    camera: CameraUniform,
    pixel00_loc: Vec3,
    _padding0: u32,
    pixel_delta_u: Vec3,
    _padding1: u32,
    pixel_delta_v: Vec3,

    max_bounces: i32,
    samples_per_pixel: i32,
    primitive_count: i32,
    _padding2: [u32; 2],
}

impl SceneData {
    fn new(
        camera: CameraUniform,
        max_bounces: i32,
        samples_per_pixel: i32,
        primitive_count: i32,
        pixel00_loc: Vec3,
        pixel_delta_u: Vec3,
        pixel_delta_v: Vec3,
    ) -> Self {
        Self {
            camera,
            pixel00_loc,
            pixel_delta_u,
            pixel_delta_v,
            max_bounces,
            samples_per_pixel,
            primitive_count,
            _padding0: Default::default(),
            _padding1: Default::default(),
            _padding2: Default::default(),
        }
    }
}
