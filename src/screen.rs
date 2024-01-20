use crate::FORMAT;
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
pub struct ScreenBindGroup {
    pub screen_bind_group: BindGroup,
}

#[derive(Resource)]
pub struct ScreenPipeline {
    pub screen_bind_group_layout: BindGroupLayout,
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

#[derive(Hash, Clone, Eq, PartialEq)]
pub struct RaytracerPipelineKey;

// TODO: I dont think this is being used... i think it should...
impl SpecializedRenderPipeline for ScreenPipeline {
    type Key = RaytracerPipelineKey;

    fn specialize(&self, _key: Self::Key) -> RenderPipelineDescriptor {
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
