use avian2d::{math::*, prelude::*};
use bevy::color::palettes::basic::RED;
use bevy::{
    asset::RenderAssetUsages, color::palettes::tailwind, ecs::entity::EntityHashSet,
    mesh::PrimitiveTopology, prelude::*,
};
use crate::gameplay::controller::plugin::PlayerControllerBundle;
use crate::gameplay::player::{aim::*, health::*, movement::*, player::{Player, PlayerStatus, Status}, stamina::*};
use crate::gameplay::inventory::inventory::Inventory;
use crate::gameplay::item::items::DetectedLoot;

const WALK_SPEED: f32 = 1.0;
const ZERO: f32 = 0.0; 

// Sizing and Physics Variables
const ACCLERATION: f32 = 500.0; 
const DAMPING: f32 = 5.0; 
const JUMP_IMPULSE: f32 = 400.0; 
const RADIUS: f32 = 30.0;

// --- BUNDLES --- 
#[derive(Bundle)]
struct PlayerBundle {
    player: Player,
    health: Health,
    inventory: Inventory,  
    mesh: Mesh2d,
    material: MeshMaterial2d<ColorMaterial>,
    speed: Speed,
    stamina: Stamina,
    status: PlayerStatus,
    transform: Transform,
    visibility: VisibilityCone, 
}

// --- ENUMS ---
#[derive(PhysicsLayer, Default)] 
pub enum Layer {
    #[default]
    Default, 
    InteractionSensor, 
    Item, 
    Player,
}

// --- COMPONENTS --- 
#[derive(Component)]
pub struct InteractionSensor; 

#[derive(Component)]
pub struct Speed {
    pub base: f32,
    pub current: f32, 
}

pub fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let shape = Circle::new(RADIUS);
    let player = commands
        .spawn((
            PlayerBundle {
                player: Player,
                health: Health {
                    max: 100.0,
                    current: 100.0,
                },
                inventory: Inventory { items: Vec::new(), searching: false }, 
                mesh: Mesh2d(meshes.add(shape)),
                material: MeshMaterial2d(materials.add(Color::from(RED))),
                speed: Speed {base: WALK_SPEED, current: WALK_SPEED},
                stamina: Stamina {
                    max: 100.0,
                    current: 100.0,
                },
                status: PlayerStatus { condition: Status::Normal, duration: ZERO},
                transform: Transform::from_xyz(100.0, 0.0, 2.0),
                visibility: VisibilityCone {
                    range: 1000.0,
                    angle: 90.0_f32.to_radians(),
                    direction: Vec2::new(0.0, 0.0),
                },
            },
            PlayerControllerBundle::new(Collider::circle(RADIUS)).with_movement(
                ACCLERATION,
                DAMPING,
                JUMP_IMPULSE,
            ),
            ColliderDensity(2.0),
            Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
            GravityScale(0.0),
            Restitution::ZERO.with_combine_rule(CoefficientCombine::Min),
        ))
        .with_children(|parent| {
            parent.spawn((
                InteractionSensor,
                Collider::circle(RADIUS * 2.0), 
                Sensor, 
                CollisionLayers::new (
                    [Layer::InteractionSensor], // Does not collide with
                    [Layer::Item] // Collides with 
                ), 
                Transform::default(),
            ));
        })
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