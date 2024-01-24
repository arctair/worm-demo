use bevy::app::{App, Startup, Update};
use bevy::DefaultPlugins;
use bevy::math::Vec2;
use bevy::prelude::{Camera2dBundle, Color, Commands, Component, Gizmos, OrthographicProjection, Query};
use bevy::utils::default;
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
        .add_systems(Update, update)
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
}

impl Polyline {
    fn collider(&self) -> Collider {
        Collider::polyline(self.points.clone(), None)
    }
}

impl From<Vec<Vec2>> for Polyline {
    fn from(points: Vec<Vec2>) -> Self {
        Polyline { points }
    }
}

fn update(
    query: Query<&Polyline>,
    mut gizmos: Gizmos,
) {
    for polyline in query.iter() {
        for point in &polyline.points {
            gizmos.circle_2d(*point, 1. / 4., Color::MAROON);
        }
    }
}