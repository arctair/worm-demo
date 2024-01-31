use bevy::app::{App, Startup, Update};
use bevy::DefaultPlugins;
use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::{Camera, Camera2dBundle, Color, Commands, Component, EventReader, Gizmos, GlobalTransform, KeyCode, OrthographicProjection, Query, Transform, TransformBundle, With};
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

        .add_systems(Startup, startup_terrain)
        .add_systems(Update, update_terrain)

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
        .insert(TransformBundle::from_transform(Transform::from_xyz(-4., 4., 0.)))
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
    action: bool,
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
            (Some(KeyCode::Space), ButtonState::Pressed) => { player_controls.action = true }
            (Some(KeyCode::Space), ButtonState::Released) => { player_controls.action = false }
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

fn startup_terrain(mut commands: Commands) {
    commands.spawn(RigidBody::Fixed)
        .insert(Polygon::from(vec![
            Vec2::new(-0.5, -0.5),
            Vec2::new(-0.5, 0.5),
            Vec2::new(0.5, 0.5),
            Vec2::new(0.5, -0.5),
        ]))
        .insert(TransformBundle::from_transform(Transform::from_xyz(32., -32., 0.).with_scale(Vec3::splat(64.))))
    ;
}

#[derive(Component)]
struct Polygon {
    vertices: Vec<Vec2>,
}

impl From<Vec<Vec2>> for Polygon {
    fn from(vertices: Vec<Vec2>) -> Self {
        Polygon { vertices }
    }
}

fn update_terrain(
    mut player_query: Query<(&Controls, &Transform), With<Player>>,
    terrain_query: Query<(&Polygon, &Transform)>,
    mut gizmos: Gizmos,
) {
    let (player_controls, player_transform) = player_query.single_mut();

    if player_controls.action {
        let right = player_transform.right();
        let up = player_transform.up();
        let offsets = vec![2. * right + 2. * up, 6. * right + 2. * up, 6. * right - 2. * up, 2. * right - 2. * up];
        for offset in offsets {
            let position = player_transform.translation + offset;
            gizmos.circle_2d(position.truncate(), 0.25, Color::YELLOW);
        }
    }

    for (polygon, transform) in terrain_query.iter() {
        for index in 0..polygon.vertices.len() {
            let start = transform.translation + transform.scale * polygon.vertices[index].extend(0.);
            let end = transform.translation + transform.scale * polygon.vertices[(index + 1) % polygon.vertices.len()].extend(0.);
            gizmos.line(start, end, Color::ORANGE);
        }
    }
}