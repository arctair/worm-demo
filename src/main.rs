use bevy::app::{App, Startup, Update};
use bevy::DefaultPlugins;
use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use bevy::math::Vec2;
use bevy::prelude::{Bundle, Camera2dBundle, Commands, Component, EventReader, KeyCode, OrthographicProjection, Query, Transform};
use bevy::transform::TransformBundle;
use bevy::utils::default;
use bevy_rapier2d::dynamics::{ExternalForce, RigidBody};
use bevy_rapier2d::geometry::Collider;
use bevy_rapier2d::plugin::{NoUserData, RapierPhysicsPlugin};
use bevy_rapier2d::render::RapierDebugRenderPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(10.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, startup)
        .add_systems(Update, (set_controls, apply_force))
        .run();
}

fn startup(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scale: 1. / 16.,
            ..default()
        },
        ..default()
    });
    commands.spawn(PlatformBundle::default());
    commands.spawn(WormBundle {
        transform_bundle: TransformBundle::from_transform(Transform::from_xyz(0., 4., 0.)),
        ..default()
    });
}

#[derive(Bundle)]
struct PlatformBundle {
    collider: Collider,
    rigid_body: RigidBody,
    transform_bundle: TransformBundle,
}

impl Default for PlatformBundle {
    fn default() -> Self {
        PlatformBundle {
            collider: Collider::cuboid(20., 1.),
            rigid_body: RigidBody::Fixed,
            transform_bundle: TransformBundle::from_transform(Transform::from_xyz(0., -1., 0.)),
        }
    }
}

#[derive(Bundle)]
struct WormBundle {
    collider: Collider,
    controls: Controls,
    external_force: ExternalForce,
    rigid_body: RigidBody,
    transform_bundle: TransformBundle,
}

impl Default for WormBundle {
    fn default() -> Self {
        WormBundle {
            collider: Collider::ball(1.0),
            controls: Default::default(),
            external_force: Default::default(),
            rigid_body: Default::default(),
            transform_bundle: Default::default(),
        }
    }
}


#[derive(Component, Default)]
struct Controls {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

fn set_controls(
    mut key_events: EventReader<KeyboardInput>,
    mut query: Query<&mut Controls>,
) {
    let mut controls = query.single_mut();

    for key_event in key_events.read() {
        match (key_event.key_code, key_event.state) {
            (Some(KeyCode::W), ButtonState::Pressed) => { controls.up = true }
            (Some(KeyCode::W), ButtonState::Released) => { controls.up = false }
            (Some(KeyCode::A), ButtonState::Pressed) => { controls.left = true }
            (Some(KeyCode::A), ButtonState::Released) => { controls.left = false }
            (Some(KeyCode::S), ButtonState::Pressed) => { controls.down = true }
            (Some(KeyCode::S), ButtonState::Released) => { controls.down = false }
            (Some(KeyCode::D), ButtonState::Pressed) => { controls.right = true }
            (Some(KeyCode::D), ButtonState::Released) => { controls.right = false }
            (_, _) => {}
        }
    }
}

fn apply_force(
    mut query: Query<(&Controls, &mut ExternalForce)>,
) {
    let (controls, mut external_force) = query.single_mut();

    let mut force = Vec2::new(0.0, 0.0);
    if controls.up { force += Vec2::Y };
    if controls.down { force += Vec2::NEG_Y };
    if controls.left { force += Vec2::NEG_X };
    if controls.right { force += Vec2::X };
    external_force.force = force.normalize_or_zero() * Vec2::new(1.0, 4.0);
}
