use bevy::app::{App, Startup, Update};
use bevy::DefaultPlugins;
use bevy::prelude::{Camera, Camera2dBundle, Commands, Component, GlobalTransform, OrthographicProjection, Query, Transform, TransformBundle, With};
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
            scale: 1.,
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
        .insert(Collider::cuboid(16., 16.))
        .insert(Player);
}

#[derive(Component)]
struct Player;

fn update_player(
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut player_query: Query<(&mut Velocity, &mut Transform), With<Player>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let (camera, camera_transform) = camera_query.single();
    let (mut player_velocity, mut player_transform) = player_query.single_mut();

    if let Some(cursor_point) = window_query
        .single()
        .cursor_position()
        .and_then(|cursor_position| camera.viewport_to_world_2d(camera_transform, cursor_position))
    {
        let old_forward = player_transform.local_y().truncate();
        let new_forward = cursor_point - player_transform.translation.truncate();
        player_transform.rotate_z(old_forward.angle_between(new_forward));
    }
}
