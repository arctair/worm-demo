use bevy::app::{App, Startup, Update};
use bevy::DefaultPlugins;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::{Camera, Camera2dBundle, Color, Commands, Component, Entity, Gizmos, GlobalTransform, OrthographicProjection, Query, Transform, TransformBundle, With};
use bevy::utils::default;
use bevy::window::{PrimaryWindow, Window};
use bevy_rapier2d::dynamics::RigidBody;
use bevy_rapier2d::geometry::Collider;
use bevy_rapier2d::plugin::{NoUserData, RapierPhysicsPlugin};
use bevy_rapier2d::render::RapierDebugRenderPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(1.))
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, startup_camera)
        .add_systems(Startup, startup_polyline)
        .add_systems(Startup, startup_player)
        .add_systems(Update, (update_player, nudge_vertices, polyline_gizmo))
        .run();
}

fn startup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scale: 1. / 32.,
            ..default()
        },
        ..default()
    });
}

fn startup_polyline(mut commands: Commands) {
    let mut points = vec![];

    {
        let width = 24;
        let count = width + 1;
        for index in 0..count {
            let x = index as f32 - (count - 1) as f32 / 2.;
            let point = Vec2::new(x, 0.);
            points.push(point)
        }
    }

    let polyline = Polyline::from(points);
    commands.spawn(RigidBody::Fixed)
        .insert(polyline.collider())
        .insert(polyline);
}

#[derive(Component)]
struct Polyline {
    points: Vec<Vec2>,
    version: usize,
}

impl Polyline {
    fn collider(&self) -> Collider {
        Collider::polyline(self.points.clone(), None)
    }
}

impl From<Vec<Vec2>> for Polyline {
    fn from(points: Vec<Vec2>) -> Self {
        Polyline { points, version: 0 }
    }
}

fn startup_player(mut commands: Commands) {
    commands.spawn(RigidBody::Fixed)
        .insert(TransformBundle::default())
        .insert(Collider::ball(1. / 4.))
        .insert(Player);
}

#[derive(Component)]
struct Player;

fn update_player(
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut player_query: Query<&mut Transform, With<Player>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let (camera, transform) = camera_query.single();
    let point = window_query.single().cursor_position()
        .and_then(|position| camera.viewport_to_world(transform, position))
        .map(|ray| ray.origin.truncate());
    if let Some(point) = point {
        let mut transform = player_query.single_mut();
        transform.translation = Vec3::new(point.x, point.y, 0.);
    }
}

fn nudge_vertices(
    mut polyline_query: Query<&mut Polyline>,
    player_query: Query<&Transform, With<Player>>,
) {
    let transform = player_query.single();
    let mut polyline = polyline_query.single_mut();

    let mut new_version = polyline.version;
    let new_points = polyline.points.iter().map(|point| {
        let distance = point.distance(transform.translation.truncate());
        if distance >= 1. / 4. {
            *point
        } else {
            new_version += 1;
            let direction = transform.looking_at(point.extend(0.), Vec3::Y).forward();
            *point + (direction * (1. / 4. - distance)).truncate()
        }
    }).collect();

    if new_version > polyline.version {
        polyline.points = new_points;
        polyline.version = new_version;
    }
}

fn polyline_gizmo(
    query: Query<&Polyline>,
    mut gizmos: Gizmos,
) {
    for polyline in query.iter() {
        for point in &polyline.points {
            gizmos.circle_2d(*point, 1. / 4., Color::MAROON);
        }
    }
}

