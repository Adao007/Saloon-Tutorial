use super::player::Player;
use bevy::color::palettes::basic::RED;
use bevy::prelude::*;

pub fn init_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

pub fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Player,
        Mesh2d(meshes.add(Triangle2d::new(
            Vec2::new(0.0, 20.0),
            Vec2::new(-20.0, -20.0),
            Vec2::new(20.0, -20.0),
        ))),
        MeshMaterial2d(materials.add(Color::from(RED))),
        Transform::from_xyz(0.0, 0.0, 1.0),
    ));
}
