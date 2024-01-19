use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use std::f32::consts::PI;
use bevy::app::{App, Startup, Update};
use bevy::DefaultPlugins;
use bevy::ecs::bundle::DynamicBundle;
use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use bevy::math::{Quat, Vec3};
use bevy::pbr::{CascadeShadowConfigBuilder, DirectionalLightBundle};
use bevy::prelude::{Camera3dBundle, Commands, Component, DirectionalLight, EventReader, KeyCode, Query, Transform};
use bevy::utils::default;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, (startup, startup_soil, startup_worm))
        .add_systems(Update, (set_controls, apply_movement_from_controls))
        .run();
}

fn startup(
    mut commands: Commands,
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
}

fn startup_soil(mut commands: Commands) {
    commands.spawn_batch(spawn_stones(256))
}

fn spawn_stones(count: usize) -> Vec<StoneBundle> {
    let p1 = 0.7548776662466927;
    let p2 = 0.5698402909980532;
    (0..count).into_iter()
        .map(|index| { Vec3::new((p1 * index as f32) % 1. - 0.5, 0., (p2 * index as f32) % 1. - 0.5) })
        .map(|normal| {
            StoneBundle {
                rigid_body: RigidBody::Fixed,
                collider: Collider::ball(0.25),
                transform_bundle: TransformBundle::from_transform(Transform::from_translation(64. * normal)),
                friction: Friction::new(0.),
                gravity_scale: GravityScale(0.),
            }
        })
        .collect()
}

#[derive(Bundle)]
struct StoneBundle {
    rigid_body: RigidBody,
    collider: Collider,
    transform_bundle: TransformBundle,
    friction: Friction,
    gravity_scale: GravityScale,
}

#[derive(Component, Default)]
struct Controls {
    scale: f32,
    segment_count: usize,
    forward: bool,
    back: bool,
    left: bool,
    right: bool,
}

fn startup_worm(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let scale = 1.;
    let segments: usize = 8;
    let transform = Transform::default();
    let mut tail = commands
        .spawn(RigidBody::Dynamic)
        .insert(TransformBundle::from_transform(transform))
        .insert(LockedAxes::TRANSLATION_LOCKED_Y | LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z)
        .insert(ExternalForce::default())
        .insert(Controls { scale, segment_count: segments, ..default() })
        .insert(Collider::ball(scale))
        .insert(Damping { linear_damping: scale * scale, angular_damping: scale * scale })
        .id();

    let camera = Camera3dBundle {
        transform: Transform::from_xyz(0., 1., 0.).looking_at(Vec3::new(0., 0., 0.), Vec3::NEG_Z),
        ..default()
    };

    let camera_joint = RevoluteJointBuilder::new(Vec3::Y)
        .local_anchor1(Vec3::new(0., segments as f32 * scale, 0.));

    commands.spawn(RigidBody::Dynamic)
        .insert(TransformBundle::from_transform(Transform::from_xyz(0., segments as f32 * scale, 0.)))
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(ImpulseJoint::new(tail, camera_joint))
        .insert(AdditionalMassProperties::Mass(0.00001))
        .with_children(|parent| { parent.spawn(camera); });

    for index in 0..segments {
        let joint = RevoluteJointBuilder::new(Vec3::Y)
            .local_anchor1(Vec3::new(0., 0., scale))
            .local_anchor2(Vec3::new(0., 0., -scale));

        tail = commands.spawn(RigidBody::Dynamic)
            .insert(TransformBundle::from_transform(Transform::from_translation(scale * 2. * (index + 1) as f32 * transform.back())))
            .insert(GravityScale(0.))
            .insert(Damping { linear_damping: scale * scale, angular_damping: scale })
            .insert(Collider::ball(scale))
            .insert(ImpulseJoint::new(tail, joint))
            .id();
    }
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
            (Some(KeyCode::A), ButtonState::Pressed) => { controls.left = true }
            (Some(KeyCode::A), ButtonState::Released) => { controls.left = false }
            (Some(KeyCode::S), ButtonState::Pressed) => { controls.back = true }
            (Some(KeyCode::S), ButtonState::Released) => { controls.back = false }
            (Some(KeyCode::D), ButtonState::Pressed) => { controls.right = true }
            (Some(KeyCode::D), ButtonState::Released) => { controls.right = false }
            (_, _) => {}
        }
    }
}

fn apply_movement_from_controls(
    mut query: Query<(&Controls, &Transform, &mut ExternalForce)>,
) {
    let (controls, transform, mut external_force) = query.single_mut();

    let mut rotation = 0.;
    if controls.left { rotation += 32. }
    if controls.right { rotation -= 32. }
    external_force.torque = Vec3::new(0., rotation, 0.);

    let mut force = Vec3::ZERO;
    if controls.forward { force += 256. * transform.forward() };
    if controls.back { force += 256. * transform.back() };
    external_force.force = force;
}
