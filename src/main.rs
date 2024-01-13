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
struct Ship {
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
        .insert(Ship::default())
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
    mut query: Query<&mut Ship>,
) {
    let mut ship = query.single_mut();

    for key_event in key_events.read() {
        match (key_event.key_code, key_event.state) {
            (Some(KeyCode::W), ButtonState::Pressed) => { ship.forward = true }
            (Some(KeyCode::W), ButtonState::Released) => { ship.forward = false }
            (Some(KeyCode::S), ButtonState::Pressed) => { ship.backward = true }
            (Some(KeyCode::S), ButtonState::Released) => { ship.backward = false }
            (Some(KeyCode::A), ButtonState::Pressed) => { ship.left = true }
            (Some(KeyCode::A), ButtonState::Released) => { ship.left = false }
            (Some(KeyCode::D), ButtonState::Pressed) => { ship.right = true }
            (Some(KeyCode::D), ButtonState::Released) => { ship.right = false }
            (Some(KeyCode::Q), ButtonState::Pressed) => { ship.rotate_left = true }
            (Some(KeyCode::Q), ButtonState::Released) => { ship.rotate_left = false }
            (Some(KeyCode::E), ButtonState::Pressed) => { ship.rotate_right = true }
            (Some(KeyCode::E), ButtonState::Released) => { ship.rotate_right = false }
            (_, _) => {}
        }
    }
}

fn apply_controls(
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &Ship, &Transform)>,
) {
    let (mut velocity, ship, transform) = query.single_mut();

    let linear_acceleration = time.delta_seconds() * 4.;
    let angular_acceleration = time.delta_seconds() * 2.;

    let forward_acceleration = linear_acceleration * transform.forward();
    let backward_acceleration = -linear_acceleration * transform.forward();
    let left_acceleration = Quat::from_rotation_y(PI / 2.) * forward_acceleration;
    let right_acceleration = Quat::from_rotation_y(-PI / 2.) * forward_acceleration;
    let yaw_acceleration = Vec3::new(0., angular_acceleration, 0.);

    if ship.forward { velocity.linvel += forward_acceleration }
    if ship.backward { velocity.linvel += backward_acceleration }
    if ship.left { velocity.linvel += left_acceleration }
    if ship.right { velocity.linvel += right_acceleration }
    if !ship.forward && !ship.backward && !ship.left && !ship.right && velocity.linvel.length() > 0. {
        let deceleration = linear_acceleration * velocity.linvel.normalize();
        velocity.linvel -= deceleration
    }
    if ship.rotate_left { velocity.angvel += yaw_acceleration }
    if ship.rotate_right { velocity.angvel -= yaw_acceleration }
    if !ship.rotate_left && !ship.rotate_right {
        let sign = velocity.angvel.y.signum();
        velocity.angvel -= sign * yaw_acceleration
    }
}