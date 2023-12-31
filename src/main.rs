use bevy::{prelude::*, render::camera::CameraRenderGraph, window::WindowPlugin};
use rusticrayz::RaytracerPlugin;

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
        camera_render_graph: CameraRenderGraph::new(rusticrayz::graph::NAME),
        camera_3d: Camera3d {
            // clear_color: Color::WHITE.into(),
            ..default()
        },
        ..default()
    },));
}
