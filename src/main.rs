use bevy::{
    core_pipeline::core_3d,
    ecs::query::WorldQuery,
    prelude::*,
    render::{
        camera::CameraRenderGraph,
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_asset::RenderAssets,
        render_graph::{self, RenderGraph, RenderGraphApp, SlotInfo, SlotType, ViewNodeRunner},
        render_resource::*,
        renderer::{RenderContext, RenderDevice},
        view::ViewTarget,
        Render, RenderApp, RenderSet,
    },
    window::WindowPlugin,
};
use std::{borrow::Cow, mem};

const SIZE: (u32, u32) = (1280, 720);
const WORKGROUP_SIZE: u32 = 8;

const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;
const COLOR_BUFFER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
const MAX_TRIANGLE_COUNT: u64 = 32;
const RENDER_SCALE: f32 = 1.0;

fn main() {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::BLACK))
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    // uncomment for unthrottled FPS
                    // present_mode: bevy::window::PresentMode::AutoNoVsync,
                    ..default()
                }),
                ..default()
            }),
            RaytracerPlugin,
        ))
        .add_systems(Startup, setup);
    bevy_mod_debugdump::print_render_graph(&mut app);
    // bevy_mod_debugdump::print_schedule_graph(&mut app, Update);
    app.run();
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera3dBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 5.0))
            .looking_at(Vec3::default(), Vec3::Y),
        camera_render_graph: CameraRenderGraph::new(graph::NAME),
        camera_3d: Camera3d {
            // clear_color: Color::WHITE.into(),
            ..default()
        },
        ..default()
    },));
}

/// Render graph constants
mod graph {
    /// Raytracer sub-graph name
    pub const NAME: &str = "raytracer";

    pub mod node {
        /// Main raytracer compute shader
        pub const RAYTRACER: &str = "raytracer_pass";
        /// Write result of RAYTRACER to screen
        pub const SCREEN: &str = "screen_pass";
    }
}

pub struct RaytracerPlugin;
impl Plugin for RaytracerPlugin {
    fn build(&self, app: &mut App) {
        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.add_systems(
            Render,
            prepare_bind_group.in_set(RenderSet::PrepareBindGroups),
        );

        // Add raytracer sub-graph
        render_app.add_render_sub_graph(graph::NAME);

        // Nodes
        render_app.add_render_graph_node::<ViewNodeRunner<RaytracerNode>>(
            graph::NAME,
            graph::node::RAYTRACER,
        );
        render_app
            .add_render_graph_node::<ViewNodeRunner<ScreenNode>>(graph::NAME, graph::node::SCREEN);

        // Edges (aka dependencies)
        render_app.add_render_graph_edge(graph::NAME, graph::node::RAYTRACER, graph::node::SCREEN);
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<RaytracerPipeline>();
        render_app.init_resource::<ScreenPipeline>();
    }
}

#[derive(Resource)]
struct RaytracerBindGroup {
    rt_bind_group: BindGroup,
}

#[derive(Resource)]
struct ScreenBindGroup {
    screen_bind_group: BindGroup,
}

fn prepare_bind_group(
    mut commands: Commands,
    rt_pipeline: Res<RaytracerPipeline>,
    screen_pipeline: Res<ScreenPipeline>,
    render_device: Res<RenderDevice>,
) {
    info!("prepare bind group");

    let (color_buffer, sampler, scene_data_buffer, objects_buffer) =
        create_assets(&render_device, RENDER_SCALE);

    let color_buffer_view = color_buffer.create_view(&wgpu::TextureViewDescriptor::default());
    // TODO: i dont think bindgroups should be created every frame D:
    let rt_bind_group = render_device.create_bind_group(
        Some("raytracer_rt_bind_group"),
        &rt_pipeline.rt_bind_group_layout,
        &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&color_buffer_view),
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
    );
    let screen_bind_group = render_device.create_bind_group(
        Some("raytracer_screen_bind_group"),
        &screen_pipeline.screen_bind_group_layout,
        &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&color_buffer_view),
            },
        ],
    );

    commands.insert_resource(RaytracerBindGroup { rt_bind_group });
    commands.insert_resource(ScreenBindGroup { screen_bind_group });
}

fn create_assets(device: &RenderDevice, render_scale: f32) -> (Texture, Sampler, Buffer, Buffer) {
    let color_buffer = create_color_buffer(device, render_scale);

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("Raytracer screen sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    let scene_data_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Scene data buffer"),
        size: mem::size_of::<SceneData>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let objects_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Objects data buffer"),
        size: MAX_TRIANGLE_COUNT * mem::size_of::<Triangle>() as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    (color_buffer, sampler, scene_data_buffer, objects_buffer)
}

fn create_color_buffer(device: &RenderDevice, scale: f32) -> Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("raytracer_color_buffer"),
        size: wgpu::Extent3d {
            width: (SIZE.0 as f32 * scale) as u32,
            height: (SIZE.1 as f32 * scale) as u32,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: COLOR_BUFFER_FORMAT,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
        view_formats: &[],
    })
}

#[derive(Resource)]
pub struct RaytracerPipeline {
    rt_bind_group_layout: BindGroupLayout,
    rt_pipeline_id: CachedComputePipelineId,
}

impl FromWorld for RaytracerPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let rt_bind_group_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("raytracer_compute_bind_group_layout"),
                entries: &[
                    // Output
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: COLOR_BUFFER_FORMAT,
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
        let rt_shader = world
            .resource::<AssetServer>()
            .load("shaders/raytracer.wgsl");
        let pipeline_cache = world.resource::<PipelineCache>();
        let rt_pipeline_id = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![rt_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: rt_shader,
            shader_defs: vec![],
            entry_point: Cow::from("main"),
        });

        Self {
            rt_bind_group_layout,
            rt_pipeline_id,
        }
    }
}

#[derive(Resource)]
pub struct ScreenPipeline {
    screen_bind_group_layout: BindGroupLayout,
    screen_pipeline_id: CachedRenderPipelineId,
}

impl FromWorld for ScreenPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let screen_bind_group_layout =
            render_device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("raytracer_screen_layout"),
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

        let screen_shader = world.resource::<AssetServer>().load("shaders/screen.wgsl");
        let pipeline_cache = world.resource::<PipelineCache>();
        let screen_pipeline_id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
            label: None,
            layout: vec![screen_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            vertex: VertexState {
                shader: screen_shader.clone(),
                shader_defs: vec![],
                entry_point: Cow::from("vs_main"),
                buffers: vec![],
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
            fragment: Some(FragmentState {
                entry_point: Cow::from("fs_main"),
                targets: vec![Some(wgpu::ColorTargetState {
                    format: FORMAT,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                shader: screen_shader,
                shader_defs: vec![],
            }),
        });

        Self {
            screen_bind_group_layout,
            screen_pipeline_id,
        }
    }
}

#[derive(Hash, Clone, Eq, PartialEq)]
pub struct RaytracerPipelineKey;

// TODO: I dont think this is being used... i think it should...
impl SpecializedRenderPipeline for ScreenPipeline {
    type Key = RaytracerPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        RenderPipelineDescriptor {
            label: None,
            layout: vec![self.screen_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            vertex: VertexState {
                shader: Handle::default(),
                shader_defs: vec![],
                entry_point: Cow::from("vs_main"),
                buffers: vec![],
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
            fragment: Some(FragmentState {
                entry_point: Cow::from("fs_main"),
                targets: vec![Some(wgpu::ColorTargetState {
                    format: FORMAT,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                shader: Handle::default(),
                shader_defs: vec![],
            }),
        }
    }
}

#[derive(Default)]
struct RaytracerNode;

impl render_graph::ViewNode for RaytracerNode {
    // ViewTargets are cameras
    type ViewQuery = &'static ViewTarget;

    fn run(
        &self,
        graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        view_query: <Self::ViewQuery as WorldQuery>::Item<'_>,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        info!("Raytracer Node run");

        let bind_groups = world.resource::<RaytracerBindGroup>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<RaytracerPipeline>();

        // Raytracer pass
        {
            let mut compute_pass = render_context
                .command_encoder()
                .begin_compute_pass(&ComputePassDescriptor::default());
            let rt_pipeline = pipeline_cache
                .get_compute_pipeline(pipeline.rt_pipeline_id)
                .unwrap();
            compute_pass.set_pipeline(rt_pipeline);
            compute_pass.set_bind_group(0, &bind_groups.rt_bind_group, &[]);
            compute_pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);
        }

        Ok(())
    }
}

#[derive(Default)]
struct ScreenNode;

impl render_graph::ViewNode for ScreenNode {
    // ViewTargets are cameras
    type ViewQuery = &'static ViewTarget;

    fn run(
        &self,
        graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        view_query: <Self::ViewQuery as WorldQuery>::Item<'_>,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        info!("Render Node run");

        let bind_groups = world.resource::<ScreenBindGroup>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<ScreenPipeline>();

        {
            let mut render_pass =
                render_context
                    .command_encoder()
                    .begin_render_pass(&RenderPassDescriptor {
                        label: Some("Raytracer render pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: view_query.out_texture(),
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: None,
                    });
            let screen_pipeline = pipeline_cache
                .get_render_pipeline(pipeline.screen_pipeline_id)
                .unwrap();
            render_pass.set_pipeline(screen_pipeline);
            render_pass.set_bind_group(0, &bind_groups.screen_bind_group, &[]);
            render_pass.draw(0..6, 0..1);
        }

        Ok(())
    }
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

#[repr(C, align(16))]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Triangle {
    corner_a: Vec3,
    _padding0: u32,
    corner_b: Vec3,
    _padding1: u32,
    corner_c: Vec3,
    _padding2: u32,

    normal_a: Vec3,
    _padding3: u32,
    normal_b: Vec3,
    _padding4: u32,
    normal_c: Vec3,
    _padding5: u32,

    color: Vec3,
    _padding6: u32,
}

#[repr(C, align(16))]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pos: Vec3,
    _padding0: u32,
    forwards: Vec3,
    _padding1: u32,
    right: Vec3,
    _padding2: u32,
    up: Vec3,
    _padding3: u32,
}
