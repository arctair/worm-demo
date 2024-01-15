use std::f32::consts::PI;
use bevy::app::{App, Startup, Update};
use bevy::asset::{Assets, Handle};
use bevy::DefaultPlugins;
use bevy::ecs::system::EntityCommands;
use bevy::hierarchy::BuildChildren;
use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use bevy::math::{Quat, Vec3};
use bevy::pbr::{CascadeShadowConfigBuilder, DirectionalLightBundle, PbrBundle, StandardMaterial};
use bevy::prelude::{Camera3dBundle, Color, Commands, Component, DirectionalLight, Entity, EventReader, KeyCode, Mesh, Query, Res, ResMut, SpatialBundle, Transform};
use bevy::prelude::shape::{Icosphere};
use bevy::time::Time;
use bevy::utils::default;


fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (startup, startup_worm))
        .add_systems(Update, (set_controls, apply_movement_from_controls))
        .add_systems(Update, worm_node_system)
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

#[derive(Component, Default)]
struct Controls {
    forward: bool,
    left: bool,
    right: bool,
}

#[derive(Component, Default)]
struct WormBody {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    translation: Option<Vec3>,
    nodes: Vec<Entity>,
}

impl WormBody {
    fn append_node(&mut self, mut entity_commands: EntityCommands, translation: Vec3) {
        self.translation = Some(translation);

        let bundle = PbrBundle {
            mesh: self.mesh.clone(),
            material: self.material.clone(),
            transform: Transform::from_translation(translation),
            ..default()
        };

        self.nodes.push(entity_commands.insert(bundle).id());
    }
}

fn startup_worm(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Mesh::try_from(Icosphere { radius: 0.25, subdivisions: 2 }).unwrap());
    let material = materials.add(Color::BEIGE.into());

    let mut body = WormBody {
        mesh: mesh.clone(),
        material: material.clone(),
        ..default()
    };

    for index in -16..0 {
        body.append_node(commands.spawn_empty(), Vec3::new(0., 0., 0.25 * index as f32));
    }

    commands.spawn_empty()
        .insert(Controls::default())
        .insert(SpatialBundle {
            transform: Transform::from_rotation(Quat::from_rotation_y(PI)),
            ..default()
        })
        .insert(body)
        .with_children(|parent| {
            parent.spawn(PbrBundle {
                mesh: mesh.clone(),
                material: material.clone(),
                ..default()
            });
            parent.spawn(Camera3dBundle {
                transform: Transform::from_xyz(0., 16., 1.).looking_at(Vec3::new(0., 0., 1.), Vec3::NEG_Z),
                ..default()
            });
        });
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
            (Some(KeyCode::D), ButtonState::Pressed) => { controls.right = true }
            (Some(KeyCode::D), ButtonState::Released) => { controls.right = false }
            (_, _) => {}
        }
    }
}

fn apply_movement_from_controls(
    time: Res<Time>,
    mut query: Query<(&Controls, &mut Transform)>,
) {
    let (controls, mut transform) = query.single_mut();

    let mut rotation = 0.;
    if controls.left { rotation += PI / 4. }
    if controls.right { rotation -= PI / 4. }

    if controls.forward {
        transform.rotate(Quat::from_rotation_y(rotation * time.delta_seconds()));

        let velocity = 2. * transform.forward();
        transform.translation += velocity * time.delta_seconds();
    }
}

fn worm_node_system(
    mut commands: Commands,
    mut query: Query<(&Transform, &mut WormBody)>,
) {
    let distance_between = 0.25;
    let max_count = 16;
    let (transform, mut body) = query.single_mut();

    match body.translation {
        None =>
            body.append_node(commands.spawn_empty(), transform.translation),
        Some(translation) if transform.translation.distance(translation) >= distance_between =>
            body.append_node(commands.spawn_empty(), translation + distance_between * (transform.translation - translation).normalize()),
        Some(_) => {}
    }

    while body.nodes.len() > max_count {
        commands.entity(body.nodes.remove(0)).despawn()
    }
}

