use std::f32::consts::SQRT_2;
use std::iter;
use bevy::app::{App, Startup, Update};
use bevy::DefaultPlugins;
use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::{Bundle, Camera2dBundle, Color, Commands, Component, EventReader, Gizmos, KeyCode, OrthographicProjection, Query, Res, Transform, With};
use bevy::prelude::shape::RegularPolygon;
use bevy::time::Time;
use bevy::transform::TransformBundle;
use bevy::utils::default;
use bevy_rapier2d::dynamics::{Damping, ExternalForce, GravityScale, RigidBody};
use bevy_rapier2d::geometry::{Collider, CollisionGroups, Group};
use bevy_rapier2d::plugin::{NoUserData, RapierPhysicsPlugin};
use bevy_rapier2d::render::RapierDebugRenderPlugin;
use geometry::Geometry;
use crate::regular_polygon::Vertices;

mod regular_polygon;
mod geometry;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(10.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, startup)
        .add_systems(Update, controls)
        .add_systems(Update, debug_subtraction)
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

    let transform = Transform::from_xyz(0., 4., 0.);
    commands.spawn(WormBundle {
        transform_bundle: TransformBundle::from_transform(transform),
        ..default()
    });

    let size = 1;
    let scale = Vec3::splat(4.);
    for x in (-size / 2)..((size + 1) / 2) {
        for y in (-size / 2)..((size + 1) / 2) {
            let translation = (1 - size % 2) as f32 * scale + 2. * scale * Vec3::new(x as f32, y as f32, 0.);
            let transform = Transform::from_translation(translation).with_scale(scale);
            commands.spawn(HunkBundle {
                transform_bundle: TransformBundle::from_transform(transform),
                ..default()
            });
        }
    }
}

impl Default for WormBundle {
    fn default() -> Self {
        let vertices = RegularPolygon { radius: 4., sides: 3 }.vertices();
        WormBundle {
            collider: Collider::convex_polyline(vertices.clone()).unwrap(),
            collision_groups: CollisionGroups::new(Group::GROUP_2, Group::GROUP_2),
            controls: Controls::default(),
            damping: Damping { linear_damping: 0.25, angular_damping: 0.0 },
            external_force: Default::default(),
            geometry: Geometry::new(vertices),
            gravity_scale: GravityScale(0.),
            rigid_body: RigidBody::Dynamic,
            transform_bundle: Default::default(),
        }
    }
}

#[derive(Bundle)]
struct WormBundle {
    collider: Collider,
    collision_groups: CollisionGroups,
    controls: Controls,
    damping: Damping,
    external_force: ExternalForce,
    geometry: Geometry,
    gravity_scale: GravityScale,
    rigid_body: RigidBody,
    transform_bundle: TransformBundle,
}

#[derive(Component, Default)]
struct Controls {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    action: bool,
}

impl Default for HunkBundle {
    fn default() -> Self {
        let vertices = RegularPolygon { radius: SQRT_2, sides: 4 }.vertices();
        HunkBundle {
            collider: Collider::convex_polyline(vertices.clone()).unwrap(),
            collision_groups: CollisionGroups::new(Group::GROUP_1, Group::GROUP_1),
            geometry: Geometry::new(vertices),
            hunk: Hunk,
            rigid_body: RigidBody::Fixed,
            transform_bundle: Default::default(),
        }
    }
}

#[derive(Bundle)]
struct HunkBundle {
    collider: Collider,
    collision_groups: CollisionGroups,
    geometry: Geometry,
    hunk: Hunk,
    rigid_body: RigidBody,
    transform_bundle: TransformBundle,
}

#[derive(Component)]
struct Hunk;

fn controls(
    mut key_events: EventReader<KeyboardInput>,
    mut query: Query<(&mut Controls, &mut ExternalForce)>,
) {
    let (mut controls, mut external_force) = query.single_mut();

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
            (Some(KeyCode::Space), ButtonState::Pressed) => { controls.action = true }
            (Some(KeyCode::Space), ButtonState::Released) => { controls.action = false }
            (_, _) => {}
        }
    }

    let mut force = Vec2::new(0.0, 0.0);
    if controls.up { force += Vec2::Y };
    if controls.down { force += Vec2::NEG_Y };
    if controls.left { force += Vec2::NEG_X };
    if controls.right { force += Vec2::X };
    external_force.force = 4. * force.normalize_or_zero();
}

fn debug_subtraction(
    time: Res<Time>,
    worm_query: Query<(&Transform, &Geometry), With<Controls>>,
    hunk_query: Query<(&Transform, &Geometry), With<Hunk>>,
    mut gizmos: Gizmos,
) {
    for hunk in hunk_query.iter() {
        let worm = worm_query.single();
        let subtraction = Geometry::subtract(hunk, worm);
        gizmos.linestrip_2d(subtraction.clone().into_iter().chain(iter::once(subtraction[0])), Color::YELLOW);

        let vertex = subtraction[(8. * time.elapsed_seconds()) as usize % subtraction.len()];
        let translation = Vec3::new(vertex.x, vertex.y, 0.);
        gizmos.circle(translation, Vec3::ZERO, 0.25, Color::GREEN);
    }
}
