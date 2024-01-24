use crate::{raytracer::RaytracerBindGroup, screen::ScreenBindGroup};
use bevy::{
    prelude::*,
    render::{
        render_graph::{RenderGraphApp, ViewNodeRunner},
        render_resource::*,
        renderer::RenderDevice,
        Render, RenderApp, RenderSet,
    },
};
use instance::InstancePlugin;
use mesh::{MeshMaterialBindGroupLayout, MeshPlugin};
use raytracer::{RaytracerBindGroupLayout, RaytracerNode, RaytracerPipeline};
use screen::{ScreenNode, ScreenPipeline};
use std::mem;

mod instance;
mod mesh;
mod raytracer;
mod screen;

/// Render graph constants
pub mod graph {
    /// Raytracer sub-graph name
    pub const NAME: &str = "raytracer";

    pub mod node {
        /// Main raytracer compute shader
        pub const RAYTRACER: &str = "raytracer_pass";
        /// Write result of RAYTRACER to screen
        pub const SCREEN: &str = "screen_pass";
    }
}

const SIZE: (u32, u32) = (1280, 720);
const WORKGROUP_SIZE: u32 = 8;

const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;
const COLOR_BUFFER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
const MAX_TRIANGLE_COUNT: u64 = 32;
const RENDER_SCALE: f32 = 1.0;

pub struct RaytracerPlugin;
impl Plugin for RaytracerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((MeshPlugin, InstancePlugin));

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
        render_app
            .init_resource::<MeshMaterialBindGroupLayout>()
            .init_resource::<RaytracerBindGroupLayout>()
            .init_resource::<RaytracerPipeline>()
            .init_resource::<ScreenPipeline>();
    }
}

fn prepare_bind_group(
    mut commands: Commands,
    screen_pipeline: Res<ScreenPipeline>,
    render_device: Res<RenderDevice>,
    rt_bind_group_layout: Res<RaytracerBindGroupLayout>,
) {
    // TODO: maybe color_buffer should be a resource?
    let (color_buffer, sampler, scene_data_buffer, objects_buffer) =
        create_assets(&render_device, RENDER_SCALE);

    let color_buffer_view = color_buffer.create_view(&wgpu::TextureViewDescriptor::default());
    // TODO: i dont think bindgroups should be created every frame D:
    let rt_bind_group = render_device.create_bind_group(
        Some("raytracer_rt_bind_group"),
        &rt_bind_group_layout,
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
