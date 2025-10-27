use bevy::prelude::*; 
use bevy::color::palettes::basic::RED; 

pub fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>, 
    mut materials: ResMut<Assets<ColorMaterial>>, 
) {
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(50.0))), 
        MeshMaterial2d(materials.add(Color::from(RED))),
        Transform::from_xyz(0.0, 0.0, 1.0),
    ));
}