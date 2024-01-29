use bevy::{
    core_pipeline::core_3d, prelude::*, render::camera::CameraRenderGraph, window::WindowPlugin,
};
use bevy_flycam::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
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
            NoCameraPlayerPlugin,
        ))
        .add_plugins(WorldInspectorPlugin::new())
        .add_systems(Startup, setup)
        .add_systems(Update, switch_camera);
    // bevy_mod_debugdump::print_render_graph(&mut app);
    // bevy_mod_debugdump::print_schedule_graph(&mut app, Update);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    // commands.spawn(PbrBundle {
    //     mesh: meshes.add(shape::Circle::new(4.0).into()),
    //     material: materials.add(Color::WHITE.into()),
    //     transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    //     ..default()
    // });
    // cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb_u8(124, 144, 255).into()),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    });
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // camera
    commands.spawn((
        Camera3dBundle {
            // transform: Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
            transform: Transform::from_xyz(0.0, 0.0, -9.0).looking_at(Vec3::ZERO, Vec3::Y),
            camera_render_graph: CameraRenderGraph::new(rusticrayz::graph::NAME),
            camera_3d: Camera3d {
                // clear_color: Color::WHITE.into(),
                ..default()
            },
            ..default()
        },
        FlyCam,
    ));
}

fn switch_camera(
    mut query: Query<(&mut Transform, &mut CameraRenderGraph)>,
    keys: Res<Input<KeyCode>>,
) {
    for (mut pos, mut cam) in &mut query {
        if keys.just_pressed(KeyCode::C) {
            if **cam == "raytracer" {
                info!("Switching to {}", core_3d::graph::NAME);
                cam.set(core_3d::graph::NAME);
            } else {
                info!("Switching to {}", rusticrayz::graph::NAME);
                cam.set(rusticrayz::graph::NAME);
            }
        }

        if keys.just_pressed(KeyCode::R) {
            info!("Resetting Camera");
            *pos = Transform::from_xyz(0.0, 0.0, -9.0).looking_at(Vec3::ZERO, Vec3::Y);
        }
    }
}
