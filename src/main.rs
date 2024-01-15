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
use bevy::prelude::shape::{Capsule, Plane};
use bevy::time::Time;
use bevy::utils::default;
use bevy_rapier3d::dynamics::{RigidBody, Velocity};
use bevy_rapier3d::plugin::{NoUserData, RapierPhysicsPlugin};
use bevy_rapier3d::render::RapierDebugRenderPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, startup)
        .add_systems(Update, (set_controls, apply_controls))
        .run();
}

#[derive(Component, Default)]
struct Controls {
    forward: bool,
    backward: bool,
    left: bool,
    right: bool,
    rotate_left: bool,
    rotate_right: bool,
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

    commands.spawn(RigidBody::Dynamic)
        .insert(Controls::default())
        .insert(Velocity {
            linvel: Vec3::ZERO,
            angvel: Vec3::ZERO,
        })
        .insert(SpatialBundle::default())
        .with_children(|parent| {
            parent.spawn(PbrBundle {
                mesh: meshes.add(Capsule::default().into()),
                material: materials.add(Color::RED.into()),
                transform: Transform::from_rotation(Quat::from_rotation_x(PI / 2.)),
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
            (Some(KeyCode::Q), ButtonState::Pressed) => { controls.rotate_left = true }
            (Some(KeyCode::Q), ButtonState::Released) => { controls.rotate_left = false }
            (Some(KeyCode::E), ButtonState::Pressed) => { controls.rotate_right = true }
            (Some(KeyCode::E), ButtonState::Released) => { controls.rotate_right = false }
            (_, _) => {}
        }
    }
}

fn apply_controls(
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &Controls, &Transform)>,
) {
    let (mut velocity, controls, transform) = query.single_mut();

    let linear_acceleration = time.delta_seconds() * 4.;
    let angular_acceleration = time.delta_seconds() * 2.;

    let forward_acceleration = linear_acceleration * transform.forward();
    let backward_acceleration = -linear_acceleration * transform.forward();
    let left_acceleration = Quat::from_rotation_y(PI / 2.) * forward_acceleration;
    let right_acceleration = Quat::from_rotation_y(-PI / 2.) * forward_acceleration;
    let yaw_acceleration = Vec3::new(0., angular_acceleration, 0.);

    if controls.forward { velocity.linvel += forward_acceleration }
    if controls.backward { velocity.linvel += backward_acceleration }
    if controls.left { velocity.linvel += left_acceleration }
    if controls.right { velocity.linvel += right_acceleration }
    if !controls.forward && !controls.backward && !controls.left && !controls.right && velocity.linvel.length() > 0. {
        let deceleration = linear_acceleration * velocity.linvel.normalize();
        velocity.linvel -= deceleration
    }
    if controls.rotate_left { velocity.angvel += yaw_acceleration }
    if controls.rotate_right { velocity.angvel -= yaw_acceleration }
    if !controls.rotate_left && !controls.rotate_right {
        let sign = velocity.angvel.y.signum();
        velocity.angvel -= sign * yaw_acceleration
    }
}