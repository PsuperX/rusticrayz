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
        .add_systems(Update, switch_camera)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Meshes
    let quad_mesh = meshes.add(Mesh::from(shape::Quad {
        size: Vec2::new(1.0, 1.0),
        flip: false,
    }));
    let sphere_mesh = meshes.add(Mesh::from(shape::UVSphere {
        radius: 1.0,
        sectors: 30,
        stacks: 30,
    }));

    // Materials
    let red_material = materials.add(StandardMaterial {
        base_color: Color::RED,
        emissive: Color::RED,
        ..default()
    });
    let blue_material = materials.add(StandardMaterial {
        base_color: Color::BLUE,
        emissive: Color::BLUE,
        ..default()
    });
    let white_material = materials.add(StandardMaterial {
        base_color: Color::ANTIQUE_WHITE,
        ..default()
    });

    // Objects

    // room
    let room_size = Vec3::new(6.0, 3.0, 8.0);
    let offset = Vec3::new(0.0, 0.0, 1.0);
    create_room(
        &mut commands,
        room_size,
        offset,
        &quad_mesh,
        &white_material,
    );

    commands.spawn(PbrBundle {
        mesh: quad_mesh.clone(),
        material: red_material.clone(),
        transform: Transform::from_translation(Vec3::new(
            -room_size.x / 2.0 + 0.01,
            room_size.y / 2.0,
            0.0,
        ))
        .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2))
        .with_scale(Vec3::new(room_size.z / 2.0, room_size.y / 2.0, 1.0)),
        ..default()
    });
    commands.spawn(PbrBundle {
        mesh: quad_mesh.clone(),
        material: blue_material.clone(),
        transform: Transform::from_translation(Vec3::new(
            room_size.x / 2.0 - 0.01,
            room_size.y / 2.0,
            0.0,
        ))
        .with_rotation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2))
        .with_scale(Vec3::new(room_size.z / 2.0, room_size.y / 2.0, 1.0)),
        ..default()
    });

    // right sphere
    let radius = 0.5;
    commands.spawn(PbrBundle {
        mesh: sphere_mesh.clone(),
        material: white_material.clone(),
        transform: Transform::from_translation(Vec3::new(1.0, radius, 0.0))
            .with_scale(Vec3::splat(radius)),
        ..default()
    });

    // left sphere
    let radius = 0.5;
    commands.spawn(PbrBundle {
        mesh: sphere_mesh.clone(),
        material: white_material.clone(),
        transform: Transform::from_translation(Vec3::new(-1.0, radius, 0.0))
            .with_scale(Vec3::splat(radius)),
        ..default()
    });

    // camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, room_size.y / 2.0, room_size.z / 2.0 - 0.1),
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
            *pos = Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y);
        }
    }
}

fn create_room(
    commands: &mut Commands,
    room_size: Vec3,
    offset: Vec3,
    quad_mesh: &Handle<Mesh>,
    material: &Handle<StandardMaterial>,
) {
    let half_size = room_size / 2.0;

    // Front
    commands.spawn(PbrBundle {
        mesh: quad_mesh.clone(),
        material: material.clone(),
        transform: Transform::from_translation(Vec3::new(0.0, half_size.y, -half_size.z) + offset)
            .with_scale(Vec3::new(room_size.x, room_size.y, 1.0)),
        ..Default::default()
    });

    // Back
    commands.spawn(PbrBundle {
        mesh: quad_mesh.clone(),
        material: material.clone(),
        transform: Transform::from_translation(Vec3::new(0.0, half_size.y, half_size.z) + offset)
            .with_rotation(Quat::from_rotation_y(std::f32::consts::PI))
            .with_scale(Vec3::new(room_size.x, room_size.y, 1.0)),
        ..Default::default()
    });

    // Left
    commands.spawn(PbrBundle {
        mesh: quad_mesh.clone(),
        material: material.clone(),
        transform: Transform::from_translation(Vec3::new(-half_size.x, half_size.y, 0.0) + offset)
            .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2))
            .with_scale(Vec3::new(room_size.z, room_size.y, 1.0)),
        ..Default::default()
    });

    // Right
    commands.spawn(PbrBundle {
        mesh: quad_mesh.clone(),
        material: material.clone(),
        transform: Transform::from_translation(Vec3::new(half_size.x, half_size.y, 0.0) + offset)
            .with_rotation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2))
            .with_scale(Vec3::new(room_size.z, room_size.y, 1.0)),
        ..Default::default()
    });

    // Top
    commands.spawn(PbrBundle {
        mesh: quad_mesh.clone(),
        material: material.clone(),
        transform: Transform::from_translation(Vec3::new(0.0, room_size.y, 0.0) + offset)
            .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2))
            .with_scale(Vec3::new(room_size.x, room_size.z, 1.0)),
        ..Default::default()
    });

    // Bottom
    commands.spawn(PbrBundle {
        mesh: quad_mesh.clone(),
        material: material.clone(),
        transform: Transform::from_translation(offset)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
            .with_scale(Vec3::new(room_size.x, room_size.z, 1.0)),
        ..Default::default()
    });
}
