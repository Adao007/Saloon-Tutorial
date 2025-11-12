use crate::gameplay::player::{aim::*, health::*, movement::*, player::Player, stamina::*};
use super::{ world::*};
use bevy::color::palettes::basic::RED;
use bevy::prelude::*;

const WALK_SPEED: f32 = 85.0;

#[derive(Component)]
pub struct Object;

pub fn init_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

pub fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let player = commands
        .spawn((
            Player { speed: 1.0 },
            Health {
                max: 100.0,
                current: 100.0,
            },
            Speed {
                base: WALK_SPEED,
                current: WALK_SPEED,
            },
            Stamina {
                max: 100.0,
                current: 100.0,
            },
            Mesh2d(meshes.add(Triangle2d::new(
                Vec2::new(0.0, 20.0),
                Vec2::new(-20.0, -20.0),
                Vec2::new(20.0, -20.0),
            ))),
            MeshMaterial2d(materials.add(Color::from(RED))),
            Transform::from_xyz(0.0, 0.0, 1.0),
            Velocity { linvel: Vec3::ZERO },
            VisibilityCone {
                range: 1000.0,
                angle: 90.0_f32.to_radians(),
                direction: Vec2::new(0.0, 1.0),
            },
        ))
        .id();

    // Spawn Health Bar for Player
    commands
        .spawn((
            Node {
                top: Val::Px(0.5),
                width: Val::Px(200.0),
                height: Val::Px(20.0),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.3, 0.3, 0.3, 0.9)),
            BorderColor::all(Color::BLACK),
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.8, 0.2, 0.2)),
                HealthBar { entity: player },
            ));
        });

    // Spawn Stamina Bar for Player
    commands
        .spawn((
            Node {
                top: Val::Px(18.5),
                width: Val::Px(200.0),
                height: Val::Px(20.0),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.3, 0.3, 0.3, 0.9)),
            BorderColor::all(Color::BLACK),
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.8, 0.2)),
                StaminaBar { entity: player },
            ));
        });
}

pub fn spawn_objects(mut commands: Commands) {
    spawn_wall(
        &mut commands,
        Vec3::new(50.0, 0.0, 2.0),
        Vec2::new(10.0, 500.0),
        Color::srgb(0.4, 0.4, 0.4),
    );

    spawn_wall(
        &mut commands,
        Vec3::new(500.0, 0.0, 2.0),
        Vec2::new(10.0, 500.0),
        Color::srgb(0.4, 0.4, 0.4),
    );

    spawn_item(
        &mut commands,
        Vec3::new(200.0, 1.0, 2.0),
        Color::srgb(1.0, 1.0, 0.0),
    );

    spawn_item(
        &mut commands,
        Vec3::new(-200.0, 1.0, 2.0),
        Color::srgb(1.0, 1.0, 0.0),
    );
}

fn spawn_wall(commands: &mut Commands, position: Vec3, size: Vec2, color: Color) {
    commands.spawn((
        Obstacle::rectangle(size.x, size.y),
        Sprite {
            color,
            custom_size: Some(size),
            ..default()
        },
        Transform::from_translation(position),
    ));
}

fn spawn_item(commands: &mut Commands, position: Vec3, color: Color) {
    commands.spawn((
        Object,
        Fog {
            discovered: false,
            visible: false,
            original: color,
        },
        Sprite {
            color: Color::srgba(0.0, 0.0, 0.0, 0.0),
            custom_size: Some(Vec2::new(80.0, 80.0)),
            ..default()
        },
        Transform::from_translation(position),
    ));
}
