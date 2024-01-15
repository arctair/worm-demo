use std::f32::consts::PI;
use bevy::app::{App, Startup, Update};
use bevy::asset::Assets;
use bevy::DefaultPlugins;
use bevy::hierarchy::BuildChildren;
use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use bevy::math::{Quat, Vec3};
use bevy::pbr::{CascadeShadowConfigBuilder, DirectionalLightBundle, PbrBundle, StandardMaterial};
use bevy::prelude::{Camera3dBundle, Color, Commands, Component, DirectionalLight, EventReader, KeyCode, Mesh, Query, Res, ResMut, SpatialBundle, Transform};
use bevy::prelude::shape::{Icosphere, Plane};
use bevy::time::Time;
use bevy::utils::default;


fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, startup)
        .add_systems(Update, (set_controls, apply_movement_from_controls))
        .run();
}

#[derive(Component, Default)]
struct Controls {
    forward: bool,
    backward: bool,
    left: bool,
    right: bool,
}

fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
        cascade_shadow_config: CascadeShadowConfigBuilder {
            first_cascade_far_bound: 4.0,
            maximum_distance: 10.0,
            ..default()
        }
            .into(),
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane::default().into()).into(),
        material: materials.add(Color::WHITE.into()),
        transform: Transform::default().with_scale(Vec3::splat(8.)),
        ..default()
    });

    commands.spawn_empty()
        .insert(Controls::default())
        .insert(SpatialBundle::default())
        .with_children(|parent| {
            parent.spawn(PbrBundle {
                mesh: meshes.add(Mesh::try_from(Icosphere { radius: 0.25, subdivisions: 2 }).unwrap()),
                material: materials.add(Color::BEIGE.into()),
                ..default()
            });
            parent.spawn(Camera3dBundle {
                transform: Transform::from_xyz(0., 2., 4.).looking_at(Vec3::new(0., 0., -4.), Vec3::Y),
                ..default()
            });
        });
}

fn set_controls(
    mut key_events: EventReader<KeyboardInput>,
    mut query: Query<&mut Controls>,
) {
    let mut controls = query.single_mut();

    for key_event in key_events.read() {
        match (key_event.key_code, key_event.state) {
            (Some(KeyCode::W), ButtonState::Pressed) => { controls.forward = true }
            (Some(KeyCode::W), ButtonState::Released) => { controls.forward = false }
            (Some(KeyCode::S), ButtonState::Pressed) => { controls.backward = true }
            (Some(KeyCode::S), ButtonState::Released) => { controls.backward = false }
            (Some(KeyCode::A), ButtonState::Pressed) => { controls.left = true }
            (Some(KeyCode::A), ButtonState::Released) => { controls.left = false }
            (Some(KeyCode::D), ButtonState::Pressed) => { controls.right = true }
            (Some(KeyCode::D), ButtonState::Released) => { controls.right = false }
            (_, _) => {}
        }
    }
}

fn apply_movement_from_controls(
    time: Res<Time>,
    mut query: Query<(&Controls, &mut Transform)>,
) {
    let (controls, mut transform) = query.single_mut();

    let mut rotation = 0.;
    if controls.left { rotation += PI / 4. }
    if controls.right { rotation -= PI / 4. }

    let mut velocity = Vec3::ZERO;
    if controls.forward { velocity += transform.forward() }
    if controls.backward { velocity -= transform.forward() }

    if velocity.length() > 0. {
        transform.rotate(Quat::from_rotation_y(rotation * time.delta_seconds()));
    }
    transform.translation += 2. * time.delta_seconds() * velocity;
}