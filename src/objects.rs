use bevy::{
    color::palettes::css::{BLUE, RED},
    prelude::*,
};
use crate::definitions::{Mass, ObjectMarker, Position, Vec3f64, Velocity};

pub struct ObjectsPlugin;

impl Plugin for ObjectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_bodies);
    }
}

fn spawn_bodies(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(10.))),
        MeshMaterial3d(materials.add(Color::from(RED))),
        Transform::from_xyz(-100., 0., 0.),
        Position(Vec3f64::new(-100., 0., 0.), Vec3f64::ZERO),
        Mass(2.5e15),
        Velocity(Vec3f64::new(10., 10., -2.), Vec3f64::ZERO),
        Name::new("Red sphere"),
        ObjectMarker,
    ));

    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(10.))),
        MeshMaterial3d(materials.add(Color::from(BLUE))),
        Transform::from_xyz(100., 0., 0.),
        Position(Vec3f64::new(100., 0., 0.), Vec3f64::ZERO),
        Mass(2.5e15),
        Velocity(Vec3f64::new(-10., -10., 2.), Vec3f64::ZERO),
        Name::new("Blue sphere"),
        ObjectMarker,
    ));
}