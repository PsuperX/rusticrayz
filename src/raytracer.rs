use crate::{
    mesh_material::{MeshMaterialBindGroup, MeshMaterialBindGroupLayout, TextureBindGroupLayout},
    view::{ViewBindGroup, ViewBindGroupLayout},
    ColorBuffer, RtSettings, COLOR_BUFFER_FORMAT, RT_SHADER_HANDLE, SIZE, WORKGROUP_SIZE,
};
use bevy::{
    ecs::query::WorldQuery,
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_graph,
        render_resource::*,
        renderer::{RenderContext, RenderDevice},
        view::{ViewTarget, ViewUniformOffset},
        Render, RenderApp, RenderSet,
    },
};
use std::borrow::Cow;

pub struct RaytracerPipelinePlugin;
impl Plugin for RaytracerPipelinePlugin {
    fn build(&self, app: &mut App) {
        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .init_resource::<SpecializedComputePipelines<RaytracerPipelineLayout>>()
                .add_systems(
                    Render,
                    queue_raytracer_pipeline_layout
                        .in_set(RenderSet::PrepareResources)
                        .before(queue_raytracer_pipeline),
                )
                .add_systems(
                    Render,
                    queue_raytracer_pipeline.in_set(RenderSet::PrepareResources),
                )
                .add_systems(
                    Render,
                    prepare_color_buffer_bind_group.in_set(RenderSet::PrepareBindGroups),
                );
        }
    }

    fn finish(&self, app: &mut App) {
        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .init_resource::<ColorBufferBindGroupLayout>()
                .init_resource::<RaytracerPipelineLayout>();
        }
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct ColorBufferBindGroupLayout(BindGroupLayout);
impl FromWorld for ColorBufferBindGroupLayout {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("rt_color_buffer_bind_group_layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::StorageTexture {
                    access: StorageTextureAccess::WriteOnly,
                    format: COLOR_BUFFER_FORMAT,
                    view_dimension: TextureViewDimension::D2,
                },
                count: None,
            }],
        });

        Self(layout)
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct ColorBufferBindGroup(BindGroup);

fn prepare_color_buffer_bind_group(
    mut commands: Commands,
    gpu_images: Res<RenderAssets<Image>>,
    color_buffer: Res<ColorBuffer>,
    render_device: Res<RenderDevice>,
    layout: Res<ColorBufferBindGroupLayout>,
) {
    let view = gpu_images.get(&**color_buffer).unwrap();
    let bind_group = render_device.create_bind_group(
        None,
        &layout,
        &BindGroupEntries::sequential((view.texture_view.into_binding(),)),
    );
    commands.insert_resource(ColorBufferBindGroup(bind_group));
}

#[derive(Resource)]
pub struct RaytracerPipelineLayout {
    mesh_material_layout: BindGroupLayout,
    texture_layout: TextureBindGroupLayout,
    color_buffer_layout: BindGroupLayout,
    view_buffer_layout: BindGroupLayout,
}

impl RaytracerPipelineLayout {
    fn get_layout(
        mesh_material_layout: &MeshMaterialBindGroupLayout,
        texture_layout: &TextureBindGroupLayout,
        color_buffer_layout: &ColorBufferBindGroupLayout,
        view_buffer_layout: &ViewBindGroupLayout,
    ) -> Self {
        Self {
            mesh_material_layout: mesh_material_layout.0.clone(),
            texture_layout: texture_layout.clone(),
            color_buffer_layout: color_buffer_layout.0.clone(),
            view_buffer_layout: view_buffer_layout.0.clone(),
        }
    }
}

impl FromWorld for RaytracerPipelineLayout {
    fn from_world(world: &mut World) -> Self {
        let mesh_material_layout = world.resource::<MeshMaterialBindGroupLayout>();
        let texture_layout = world.resource::<TextureBindGroupLayout>();
        let color_buffer_layout = world.resource::<ColorBufferBindGroupLayout>();
        let view_buffer_layout = world.resource::<ViewBindGroupLayout>();
        Self::get_layout(
            mesh_material_layout,
            texture_layout,
            color_buffer_layout,
            view_buffer_layout,
        )
    }
}

fn queue_raytracer_pipeline_layout(
    mut layout: ResMut<RaytracerPipelineLayout>,
    mesh_material_layout: Res<MeshMaterialBindGroupLayout>,
    texture_layout: Res<TextureBindGroupLayout>,
    color_buffer_layout: Res<ColorBufferBindGroupLayout>,
    view_buffer_layout: Res<ViewBindGroupLayout>,
) {
    *layout = RaytracerPipelineLayout::get_layout(
        &mesh_material_layout,
        &texture_layout,
        &color_buffer_layout,
        &view_buffer_layout,
    );
}

#[derive(Hash, Clone, Eq, PartialEq)]
pub struct RaytracerPipelineKey {
    max_bounces: u32,
    texture_count: u32,
}

impl RaytracerPipelineKey {
    fn new(max_bounces: u32, texture_count: u32) -> Self {
        Self {
            max_bounces,
            texture_count: texture_count.next_power_of_two(),
        }
    }
}

impl SpecializedComputePipeline for RaytracerPipelineLayout {
    type Key = RaytracerPipelineKey;

    fn specialize(&self, key: Self::Key) -> ComputePipelineDescriptor {
        ComputePipelineDescriptor {
            label: Some(Cow::Borrowed("rt_compute_pipeline")),
            layout: vec![
                self.color_buffer_layout.clone(),
                self.mesh_material_layout.clone(),
                self.texture_layout.layout.clone(),
                self.view_buffer_layout.clone(),
            ],
            push_constant_ranges: vec![],
            shader: RT_SHADER_HANDLE.clone(),
            shader_defs: vec![ShaderDefVal::UInt("MAX_BOUNCES".into(), key.max_bounces)],
            entry_point: Cow::from("main"),
        }
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct RaytracerPipeline(CachedComputePipelineId);

fn queue_raytracer_pipeline(
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedComputePipelines<RaytracerPipelineLayout>>,
    rt_pipeline_layout: Res<RaytracerPipelineLayout>,
    settings: Res<RtSettings>,
) {
    let key = RaytracerPipelineKey::new(
        settings.max_bounces,
        rt_pipeline_layout.texture_layout.texture_count,
    );
    let pipeline_id = pipelines.specialize(&pipeline_cache, &rt_pipeline_layout, key);
    commands.insert_resource(RaytracerPipeline(pipeline_id));
}

#[derive(Default)]
pub struct RaytracerNode;
impl render_graph::ViewNode for RaytracerNode {
    // ViewTargets are cameras
    type ViewQuery = (&'static ViewTarget, &'static ViewUniformOffset);

    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        (_target, view_uniform_offset): <Self::ViewQuery as WorldQuery>::Item<'_>,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let color_buffer_bind_group = world.resource::<ColorBufferBindGroup>();
        let mesh_material_bind_group = world.resource::<MeshMaterialBindGroup>();
        let view_bind_group = world.resource::<ViewBindGroup>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<RaytracerPipeline>();

        let mut compute_pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());
        if let Some(rt_pipeline) = pipeline_cache.get_compute_pipeline(**pipeline) {
            compute_pass.set_pipeline(rt_pipeline);
            compute_pass.set_bind_group(0, color_buffer_bind_group, &[]);
            compute_pass.set_bind_group(1, &mesh_material_bind_group.mesh_material, &[]);
            compute_pass.set_bind_group(2, &mesh_material_bind_group.textures, &[]);
            compute_pass.set_bind_group(3, view_bind_group, &[view_uniform_offset.offset]);
            compute_pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);
        }

        Ok(())
    }
}
