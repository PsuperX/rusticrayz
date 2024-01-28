use bevy::{
    prelude::*,
    render::{
        render_resource::*,
        renderer::RenderDevice,
        view::{ViewUniform, ViewUniforms},
        Render, RenderApp, RenderSet,
    },
};

pub struct ViewPlugin;
impl Plugin for ViewPlugin {
    fn build(&self, app: &mut App) {
        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.add_systems(
                Render,
                queue_view_bind_group.in_set(RenderSet::PrepareBindGroups),
            );
        }
    }

    fn finish(&self, app: &mut App) {
        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.init_resource::<ViewBindGroupLayout>();
        }
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct ViewBindGroupLayout(pub BindGroupLayout);
impl FromWorld for ViewBindGroupLayout {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("view_bind_group_layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: Some(ViewUniform::min_size()),
                },
                count: None,
            }],
        });

        Self(layout)
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct ViewBindGroup(BindGroup);

fn queue_view_bind_group(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    view_uniforms: Res<ViewUniforms>,
    layout: Res<ViewBindGroupLayout>,
) {
    if let Some(view_binding) = view_uniforms.uniforms.binding() {
        let bind_group = render_device.create_bind_group(
            "view_bind_group",
            &layout,
            &BindGroupEntries::single(view_binding),
        );

        commands.insert_resource(ViewBindGroup(bind_group))
    }
}
