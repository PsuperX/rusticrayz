use crate::{
    mesh_material::{MeshMaterialBindGroup, MeshMaterialBindGroupLayout},
    COLOR_BUFFER_FORMAT, SIZE, WORKGROUP_SIZE,
};
use bevy::{
    ecs::query::WorldQuery,
    prelude::*,
    render::{
        render_graph::{self},
        render_resource::*,
        renderer::{RenderContext, RenderDevice},
        view::ViewTarget,
    },
};
use std::borrow::Cow;

#[derive(Resource)]
pub struct RaytracerBindGroup {
    pub rt_bind_group: BindGroup,
}

#[derive(Resource, Deref, DerefMut)]
pub struct RaytracerBindGroupLayout(BindGroupLayout);
impl FromWorld for RaytracerBindGroupLayout {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("raytracer_compute_bind_group_layout"),
            entries: &[
                // Output
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: COLOR_BUFFER_FORMAT,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                // Scene Data
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Objects Data
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        Self(layout)
    }
}

#[derive(Resource)]
pub struct RaytracerPipeline {
    rt_pipeline_id: CachedComputePipelineId,
}

impl FromWorld for RaytracerPipeline {
    fn from_world(world: &mut World) -> Self {
        let rt_shader = world
            .resource::<AssetServer>()
            .load("shaders/raytracer.wgsl");
        let pipeline_cache = world.resource::<PipelineCache>();
        let rt_bind_group_layout = world.resource::<RaytracerBindGroupLayout>();
        let mesh_material_layout = world.resource::<MeshMaterialBindGroupLayout>();
        let rt_pipeline_id = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some(Cow::Borrowed("rt_compute_pipeline")),
            layout: vec![
                (*rt_bind_group_layout).clone(),
                (*mesh_material_layout).clone(),
            ],
            push_constant_ranges: Vec::new(),
            shader: rt_shader,
            shader_defs: vec![],
            entry_point: Cow::from("main"),
        });

        Self { rt_pipeline_id }
    }
}

#[derive(Default)]
pub struct RaytracerNode;

impl render_graph::ViewNode for RaytracerNode {
    // ViewTargets are cameras
    type ViewQuery = &'static ViewTarget;

    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        _view_query: <Self::ViewQuery as WorldQuery>::Item<'_>,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let mesh_material_bind_group = world.resource::<MeshMaterialBindGroup>();
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
            compute_pass.set_bind_group(1, &mesh_material_bind_group.mesh_material, &[]);
            compute_pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);
        }

        Ok(())
    }
}
