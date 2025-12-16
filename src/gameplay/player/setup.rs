use avian2d::{math::*, prelude::*};
use bevy::color::palettes::basic::RED;
use bevy::{
    asset::RenderAssetUsages, color::palettes::tailwind, ecs::entity::EntityHashSet,
    mesh::PrimitiveTopology, prelude::*,
};
use crate::gameplay::player::{aim::*, health::*, movement::*, player::{Player, PlayerStatus, Status}, stamina::*};
use crate::gameplay::inventory::inventory::Inventory;

const WALK_SPEED: f32 = 85.0;
const ZERO: f32 = 0.0; 

// --- BUNDLES --- 
#[derive(Bundle)]
struct PlayerBundle {
    player: Player, 
    mesh: Mesh2d,
    material: MeshMaterial2d<ColorMaterial>,
    collider: Collider,
    rigid_body: RigidBody,
    interpolation: TransformInterpolation,
    touched: TouchedEntities,
    colliding: CollidingEntities,
    status: PlayerStatus,
    health: Health,
    inventory: Inventory, 
    speed: Speed, 
    stamina: Stamina,
    transform: Transform,
    velocity: Velocity,
    visibility: VisibilityCone, 
}

pub fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let shape = Circle::new(30.0);
    let player = commands
        .spawn( PlayerBundle {
            player: Player,
            mesh: Mesh2d(meshes.add(shape)),
            material: MeshMaterial2d(materials.add(Color::from(RED))),
            collider: Collider::from(shape),
            rigid_body: RigidBody::Kinematic,
            interpolation: TransformInterpolation,
            touched: TouchedEntities::default(),
            colliding: CollidingEntities::default(), 
            status: PlayerStatus { condition: Status::Normal, duration: ZERO},
            health: Health {
                max: 100.0,
                current: 100.0,
            },
            inventory: Inventory { items: Vec::new(), searching: false }, 
            speed: Speed {
                base: WALK_SPEED,
                current: WALK_SPEED,
            },
            stamina: Stamina {
                max: 100.0,
                current: 100.0,
            },
            transform: Transform::from_xyz(0.0, 0.0, 2.0),
            velocity: Velocity { linvel: Vec3::ZERO },
            visibility: VisibilityCone {
                range: 1000.0,
                angle: 90.0_f32.to_radians(),
                direction: Vec2::new(0.0, 0.0),
            },
    }).id();

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