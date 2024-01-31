use bevy::app::{App, Startup, Update};
use bevy::DefaultPlugins;
use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use bevy::math::Vec2;
use bevy::prelude::{Camera, Camera2dBundle, Commands, Component, EventReader, GlobalTransform, KeyCode, OrthographicProjection, Query, Transform, TransformBundle, With};
use bevy::utils::default;
use bevy::window::{PrimaryWindow, Window};
use bevy_rapier2d::dynamics::RigidBody;
use bevy_rapier2d::geometry::Collider;
use bevy_rapier2d::plugin::{NoUserData, RapierPhysicsPlugin};
use bevy_rapier2d::prelude::{GravityScale, Velocity};
use bevy_rapier2d::render::RapierDebugRenderPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(1.))
        .add_plugins(RapierDebugRenderPlugin::default())

        .add_systems(Startup, startup_camera)
        .add_systems(Startup, startup_player)
        .add_systems(Update, update_player)

        .run();
}

fn startup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scale: 1. / 16.,
            ..default()
        },
        ..default()
    });
}

fn startup_player(mut commands: Commands) {
    commands.spawn(RigidBody::Dynamic)
        .insert(TransformBundle::from_transform(Transform::from_xyz(0., 0., 0.)))
        .insert(GravityScale(0.))
        .insert(Velocity::default())
        .insert(Collider::cuboid(2., 2.))
        .insert(Controls::default())
        .insert(Player);
}

#[derive(Component, Default)]
struct Controls {
    left: bool,
    right: bool,
}

#[derive(Component)]
struct Player;

fn update_player(
    mut keyboard_events: EventReader<KeyboardInput>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut player_query: Query<(&mut Controls, &mut Velocity, &mut Transform), With<Player>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let (camera, camera_transform) = camera_query.single();
    let (mut player_controls, mut player_velocity, mut player_transform) = player_query.single_mut();

    for keyboard_event in keyboard_events.read() {
        match (keyboard_event.key_code, keyboard_event.state) {
            (Some(KeyCode::A), ButtonState::Pressed) => { player_controls.left = true }
            (Some(KeyCode::A), ButtonState::Released) => { player_controls.left = false }
            (Some(KeyCode::D), ButtonState::Pressed) => { player_controls.right = true }
            (Some(KeyCode::D), ButtonState::Released) => { player_controls.right = false }
            _ => {}
        }
    }

    let left = if player_controls.left { Vec2::NEG_X } else { Vec2::ZERO };
    let right = if player_controls.right { Vec2::X } else { Vec2::ZERO };
    player_velocity.linvel = 16. * (left + right);

    if let Some(cursor_point) = window_query
        .single()
        .cursor_position()
        .and_then(|cursor_position| camera.viewport_to_world_2d(camera_transform, cursor_position))
    {
        let old_forward = player_transform.right().truncate();
        let new_forward = cursor_point - player_transform.translation.truncate();
        player_transform.rotate_z(old_forward.angle_between(new_forward));
    }
}
