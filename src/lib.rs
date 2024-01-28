use bevy::{
    asset::load_internal_asset,
    prelude::*,
    render::{
        extract_resource::*,
        render_graph::{RenderGraphApp, ViewNodeRunner},
        render_resource::*,
        RenderApp,
    },
};
use mesh_material::MeshMaterialPlugin;
use raytracer::{RaytracerNode, RaytracerPipelinePlugin};
use screen::{ScreenNode, ScreenPlugin};
use view::ViewPlugin;

mod mesh_material;
mod raytracer;
mod screen;
mod view;

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

const FORMAT: TextureFormat = TextureFormat::Bgra8UnormSrgb;
const COLOR_BUFFER_FORMAT: TextureFormat = TextureFormat::Rgba8Unorm;
const DEFAULT_MAX_BOUNCES: u32 = 1;
const DEFAULT_RENDER_SCALE: f32 = 1.0;

const RT_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(108718554336535632810954);
const SCREEN_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(8520478187035914832103433315);

pub struct RaytracerPlugin;
impl Plugin for RaytracerPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            RT_SHADER_HANDLE,
            "shaders/raytracer.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            SCREEN_SHADER_HANDLE,
            "shaders/screen.wgsl",
            Shader::from_wgsl
        );

        app.init_resource::<RtSettings>()
            .add_plugins(ExtractResourcePlugin::<RtSettings>::default())
            .add_plugins(ExtractResourcePlugin::<ColorBuffer>::default())
            .add_plugins((
                MeshMaterialPlugin,
                ViewPlugin,
                RaytracerPipelinePlugin,
                ScreenPlugin,
            ))
            .add_systems(Startup, create_color_buffer);

        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

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
}

// TODO: React to changes
#[derive(Resource, Clone, ExtractResource)]
pub struct RtSettings {
    pub max_bounces: u32,
    pub render_scale: f32,
}
impl FromWorld for RtSettings {
    fn from_world(_world: &mut World) -> Self {
        Self {
            max_bounces: DEFAULT_MAX_BOUNCES,
            render_scale: DEFAULT_RENDER_SCALE,
        }
    }
}

#[derive(Resource, Clone, ExtractResource, Deref, DerefMut)]
pub struct ColorBuffer(Handle<Image>);

// TODO: every camera should have its own color buffer... i think
fn create_color_buffer(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    settings: Res<RtSettings>,
) {
    let mut image = Image::new_fill(
        Extent3d {
            width: (SIZE.0 as f32 * settings.render_scale) as u32,
            height: (SIZE.1 as f32 * settings.render_scale) as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        COLOR_BUFFER_FORMAT,
    );
    image.texture_descriptor.usage =
        TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;
    let image = images.add(image);

    commands.insert_resource(ColorBuffer(image));
}
