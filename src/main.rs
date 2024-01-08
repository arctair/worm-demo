use std::f32::consts::PI;
use bevy::a11y::AccessibilitySystem::Update;
use bevy::app::{App, Startup};
use bevy::asset::Assets;
use bevy::DefaultPlugins;
use bevy::math::{Quat, Vec3};
use bevy::pbr::{CascadeShadowConfigBuilder, DirectionalLightBundle, PbrBundle, StandardMaterial};
use bevy::prelude::{Camera3dBundle, Color, Commands, Component, DirectionalLight, Mesh, Query, ResMut, Transform};
use bevy::prelude::shape::{Capsule, Plane};
use bevy::transform::TransformBundle;
use bevy::utils::default;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, startup)
        .run();
}

#[derive(Component)]
struct Ship;

fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-64., 32., -32.).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

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
        transform: Transform::default().with_scale(Vec3::splat(32.)),
        ..default()
    });

    commands.spawn_empty()
        .insert(
            PbrBundle {
                mesh: meshes.add(Capsule::default().into()).into(),
                material: materials.add(Color::RED.into()),
                transform: Transform::default(),
                ..default()
            }
        )
        .insert(TransformBundle::from(
            Transform::from_xyz(0., 1., 0.)
                .with_rotation(Quat::from_rotation_x(PI / 2.))
        ));
}