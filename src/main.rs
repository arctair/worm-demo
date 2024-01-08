use bevy::app::{App, Startup};
use bevy::asset::Assets;
use bevy::DefaultPlugins;
use bevy::math::Vec3;
use bevy::prelude::{Camera2dBundle, Color, ColorMaterial, Commands, Component, Mesh, ResMut, shape, Transform};
use bevy::sprite::MaterialMesh2dBundle;
use bevy::utils::default;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, startup)
        .run();
}

#[derive(Component)]
struct Tree;

fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    let p1 = 0.7548776662466927;
    let p2 = 0.5698402909980532;
    for index in 0..512 {
        let x = (p1 * index as f32) % 1. * 512. - 256.;
        let y = (p2 * index as f32) % 1. * 512. - 256.;
        commands.spawn((Tree, MaterialMesh2dBundle {
            mesh: meshes.add(shape::RegularPolygon::new(8., 8).into()).into(),
            material: materials.add(ColorMaterial::from(Color::RED)),
            transform: Transform::default().with_translation(Vec3::new(x, y, 0.)),
            ..default()
        }));
    }
}