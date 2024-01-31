mod polyline;

use bevy::app::{App, Startup, Update};
use bevy::core::Zeroable;
use bevy::DefaultPlugins;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::{Camera, Camera2dBundle, Commands, Component, Entity, GlobalTransform, OrthographicProjection, Query, Transform, TransformBundle, With};
use bevy::utils::default;
use bevy::window::{PrimaryWindow, Window};
use bevy_rapier2d::dynamics::{AdditionalMassProperties, ImpulseJoint, LockedAxes, RigidBody};
use bevy_rapier2d::geometry::{Collider, CollisionGroups};
use bevy_rapier2d::math::Vect;
use bevy_rapier2d::plugin::{NoUserData, RapierPhysicsPlugin};
use bevy_rapier2d::prelude::{GravityScale, Group, PrismaticJointBuilder, Velocity};
use bevy_rapier2d::render::RapierDebugRenderPlugin;
use crate::polyline::{Polyline, polyline_gizmo, PolylineBundle};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(1.))
        .add_plugins(RapierDebugRenderPlugin::default())

        .add_systems(Startup, startup_camera)
        .add_systems(Startup, startup_player)
        .add_systems(Update, update_player)

        .add_systems(Startup, startup_polyline)
        .add_systems(Update, (nudge_vertices, polyline_gizmo))
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
        let height = 24;
        let count = height + 1;
        for index in 0..count {
            let y = index as f32 - (count - 1) as f32 / 2.;
            let point = Vec2::new(0., y);
            points.push(point)
        }
    }

    commands.spawn(PolylineBundle::from(points));
}

fn startup_player(mut commands: Commands) {
    let player = commands.spawn(RigidBody::Dynamic)
        .insert(TransformBundle::from_transform(Transform::from_xyz(-8., 0., 0.)))
        .insert(GravityScale(0.))
        .insert(Velocity::default())
        .insert(Collider::ball(1.))
        // .insert(Damping { linear_damping: 0., angular_damping: 1. })
        .insert(Player)
        .id();

    let up_joint = PrismaticJointBuilder::new(Vect::Y)
        .local_anchor1(Vec2::new(0., 8.))
        .local_anchor2(Vec2::new(0., 0.));

    commands.spawn(RigidBody::Dynamic)
        .insert(LockedAxes::TRANSLATION_LOCKED_Y)
        .insert(AdditionalMassProperties::Mass(0.1))
        .insert(GravityScale(0.))
        .insert(Collider::polyline(vec![Vec2::new(-1., 0.), Vec2::new(1., 0.)], None))
        .insert(TransformBundle::from_transform(Transform::from_xyz(-8., 8., 0.)))
        .insert(ImpulseJoint::new(player, up_joint))
        .insert(CollisionGroups::new(Group::GROUP_2, Group::GROUP_2))
    ;

    let down_joint = PrismaticJointBuilder::new(Vect::Y)
        .local_anchor1(Vec2::new(0., 8.))
        .local_anchor2(Vec2::new(0., 0.));

    commands.spawn(RigidBody::Dynamic)
        .insert(LockedAxes::TRANSLATION_LOCKED_Y)
        .insert(AdditionalMassProperties::Mass(0.1))
        .insert(GravityScale(0.))
        .insert(Collider::polyline(vec![Vec2::new(-1., 0.), Vec2::new(1., 0.)], None))
        .insert(TransformBundle::from_transform(Transform::from_xyz(-8., -8., 0.)))
        .insert(ImpulseJoint::new(player, down_joint))
        .insert(CollisionGroups::new(Group::GROUP_2, Group::GROUP_2))
    ;
}

#[derive(Component)]
struct Player;

fn update_player(
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut player_query: Query<(&mut Velocity, &Transform), With<Player>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let (camera, transform) = camera_query.single();
    let point = window_query.single().cursor_position()
        .and_then(|position| camera.viewport_to_world(transform, position))
        .map(|ray| ray.origin.truncate());
    let (mut velocity, transform) = player_query.single_mut();

    velocity.linvel = match point {
        Some(point)  if point.distance(transform.translation.truncate()) > 1. => {
            4. * transform.looking_at(point.extend(0.), Vec3::Y).forward().truncate()
        }
        _ => Vec2::zeroed()
    }
}

fn nudge_vertices(
    mut commands: Commands,
    mut polyline_query: Query<(Entity, &mut Polyline)>,
    player_query: Query<&Transform, With<Player>>,
) {
    let distance_from_player_at_least = 1. + 1. / 8. + 1. / 8.;
    let distance_at_least = 0.5;
    let distance_at_most = 1.;

    let transform = player_query.single();
    for (entity, mut polyline) in polyline_query.iter_mut() {
        let mut new_version = polyline.version;
        let mut new_points = vec![];
        for point in &polyline.points {
            let distance_from_player = point.distance(transform.translation.truncate());
            let mut new_point = *point;
            if distance_from_player < distance_from_player_at_least {
                new_version += 1;

                let direction = transform.looking_at(point.extend(0.), Vec3::Y).forward();
                let delta = direction * (distance_from_player_at_least - distance_from_player);
                new_point += delta.truncate()
            }

            let mut last_option = new_points.last();
            if last_option.is_some_and(|last_point| point.distance(*last_point) < distance_at_least) {
                continue;
            }

            while last_option.is_some_and(|last| point.distance(*last) > distance_at_most) {
                new_version += 1;

                let last = *last_option.unwrap();
                let delta = distance_at_least * (*point - last).normalize();
                new_points.push(last + delta);

                last_option = new_points.last();
            }

            // TODO: if a shorter path to this point than new_points exists then
            // TODO:   move the path after that path to a new polyline

            for (index, check_point) in new_points.iter().enumerate() {
                if index < new_points.len() - 1 && new_point.distance(*check_point) <= distance_at_least {
                    let move_count = new_points.len() - index - 1;
                    let mut oxbow = vec![];
                    for _ in 0..move_count {
                        oxbow.push(new_points.remove(index + 1));
                    }
                    oxbow.push(oxbow[0]);
                    if oxbow.len() > 2 {
                        commands.spawn(PolylineBundle::from(oxbow.to_vec()));
                    }
                    break;
                }
            }

            new_points.push(new_point)
        }

        if new_points.len() < 3 {
            commands.entity(entity).despawn();
        } else if new_version > polyline.version {
            polyline.points = new_points;
            polyline.version = new_version;

            let mut entity = commands.entity(entity);
            entity.remove::<Collider>();
            entity.insert(polyline.collider());
        }
    }
}
