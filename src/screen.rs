use crate::{ColorBuffer, FORMAT, SCREEN_SHADER_HANDLE};
use bevy::{
    ecs::query::WorldQuery,
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_graph,
        render_resource::*,
        renderer::{RenderContext, RenderDevice},
        view::ViewTarget,
        Render, RenderApp, RenderSet,
    },
};
use std::borrow::Cow;

pub struct ScreenPlugin;
impl Plugin for ScreenPlugin {
    fn build(&self, app: &mut App) {
        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.add_systems(
                Render,
                prepare_screen_bind_group.in_set(RenderSet::PrepareBindGroups),
            );
        }
    }

    fn finish(&self, app: &mut App) {
        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .init_resource::<ScreenBindGroupLayout>()
                .init_resource::<ScreenPipeline>();
        }
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct ScreenBindGroupLayout(BindGroupLayout);
impl FromWorld for ScreenBindGroupLayout {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("raytracer_screen_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

        Self(layout)
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct ScreenBindGroup(BindGroup);

fn prepare_screen_bind_group(
    mut commands: Commands,
    gpu_images: Res<RenderAssets<Image>>,
    color_buffer: Res<ColorBuffer>,
    render_device: Res<RenderDevice>,
    layout: Res<ScreenBindGroupLayout>,
) {
    let view = gpu_images.get(&**color_buffer).unwrap();
    let bind_group = render_device.create_bind_group(
        None,
        &layout,
        &BindGroupEntries::sequential((
            view.sampler.into_binding(),
            view.texture_view.into_binding(),
        )),
    );
    commands.insert_resource(ScreenBindGroup(bind_group));
}

#[derive(Resource, Clone, Deref, DerefMut)]
pub struct ScreenPipeline(CachedRenderPipelineId);
impl FromWorld for ScreenPipeline {
    fn from_world(world: &mut World) -> Self {
        let layout = world.resource::<ScreenBindGroupLayout>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline_id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
            label: Some(Cow::Borrowed("raytracer_screen_pipeline")),
            layout: vec![layout.0.clone()],
            push_constant_ranges: vec![],
            vertex: VertexState {
                shader: SCREEN_SHADER_HANDLE.clone(),
                shader_defs: vec![],
                entry_point: Cow::from("vs_main"),
                buffers: vec![],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(FragmentState {
                entry_point: Cow::from("fs_main"),
                targets: vec![Some(ColorTargetState {
                    format: FORMAT,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
                shader: SCREEN_SHADER_HANDLE.clone(),
                shader_defs: vec![],
            }),
        });

        Self(pipeline_id)
    }
}

#[derive(Default)]
pub struct ScreenNode;
impl render_graph::ViewNode for ScreenNode {
    // ViewTargets are cameras
    type ViewQuery = &'static ViewTarget;

    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        view_query: <Self::ViewQuery as WorldQuery>::Item<'_>,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let screen_bind_group = world.resource::<ScreenBindGroup>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<ScreenPipeline>();

        let mut render_pass =
            render_context
                .command_encoder()
                .begin_render_pass(&RenderPassDescriptor {
                    label: Some("raytracer_render_pass"),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: view_query.out_texture(),
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Load,
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                });

        if let Some(screen_pipeline) = pipeline_cache.get_render_pipeline(**pipeline) {
            render_pass.set_pipeline(screen_pipeline);
            render_pass.set_bind_group(0, screen_bind_group, &[]);
            render_pass.draw(0..6, 0..1);
        }

        Ok(())
    }
}
