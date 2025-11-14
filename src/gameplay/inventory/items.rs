use std::sync::OnceLock;
use bevy::{prelude::*};
use bevy_common_assets::ron::RonAssetPlugin;
use crate::gameplay::player::{
    aim::{MousePos, get_mouse_position},
    player::Player,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::inventory::*;

pub struct ItemsPlugin; 
impl Plugin for ItemsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(RonAssetPlugin::<ItemDefinition>::new(&[".ron"]))
            .add_systems(Startup, spawn_bandage);
    }
}

#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct ItemDefinition {
    pub name: String,
    pub item_type: ItemType,
    pub description: String,
    pub max_stack: u32,
    pub shape: Shape,
    pub rotate: bool,
    pub icon: String,

    #[serde(skip)]
    rotated_cells: OnceLock<[Vec<IVec2>; 4]>,
}

impl ItemDefinition {
    pub fn get_cells(&self, rotation: Rotation) -> &[IVec2] {
        // Compute all rotations once on first access
        self.rotated_cells.get_or_init(|| {
            [
                self.shape.to_cells(&Rotation::Zero),
                if self.rotate { self.shape.to_cells(&Rotation::Ninety) } else { vec![] },
                if self.rotate { self.shape.to_cells(&Rotation::OneEighty) } else { vec![] },
                if self.rotate { self.shape.to_cells(&Rotation::TwoSeventy) } else { vec![] },
            ]
        })
        [rotation as usize]
        .as_slice()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum ItemType {
    Ammo,
    Consumable,
    Equipment,
    Salvage,
    Quest,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub enum Rotation {
    Zero,
    Ninety,
    OneEighty,
    TwoSeventy,
}

impl Rotation {
    pub fn next(&self) -> Self {
        match self {
            Rotation::Zero => Rotation::Ninety, 
            Rotation::Ninety => Rotation::OneEighty,
            Rotation::OneEighty => Rotation::TwoSeventy, 
            Rotation::TwoSeventy => Rotation::Zero,
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
pub enum Shape {
    Grid {
        width: u32,
        height: u32,
        pattern: Vec<Vec<u8>>,
    },
}

impl Shape {
     pub fn to_cells(&self, rotation: &Rotation) -> Vec<IVec2> {
        let Shape::Grid { width, height, pattern } = self;
        
        // Validate pattern matches declared dimensions
        assert_eq!(pattern.len() as u32, *height, "RON file error: pattern height mismatch");
        for (i, row) in pattern.iter().enumerate() {
            assert_eq!(row.len() as u32, *width, 
                "RON file error: pattern width mismatch on row {}", i);
        }
        
        // Collect occupied cells
        let mut cells = Vec::new();
        for (y, row) in pattern.iter().enumerate() {
            for (x, &cell) in row.iter().enumerate() {
                if cell == 1 {
                    cells.push(IVec2::new(x as i32, y as i32));
                }
            }
        }
        
        // Apply rotation
        self.rotate_cells(&cells, rotation)
    }
    
    fn rotate_cells(&self, cells: &[IVec2], rotation: &Rotation) -> Vec<IVec2> {
        let rotated: Vec<IVec2> = cells.iter().map(|pos| {
            match rotation {
                Rotation::Zero => *pos,
                Rotation::Ninety => IVec2::new(-pos.y, pos.x),
                Rotation::OneEighty => IVec2::new(-pos.x, -pos.y),
                Rotation::TwoSeventy => IVec2::new(pos.y, -pos.x),
            }
        }).collect();
        
        // Normalize so min cell is at (0,0)
        let min_x = rotated.iter().map(|p| p.x).min().unwrap_or(0);
        let min_y = rotated.iter().map(|p| p.y).min().unwrap_or(0);
        rotated.into_iter().map(|p| p - IVec2::new(min_x, min_y)).collect()
    }
}

// --- COMPONENTS --- 
#[derive(Component)]
pub struct Item {
    pub definition: Handle<ItemDefinition>,
    pub quantity: u32,
    //pub placement: Option<ItemPlacement>,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct ItemPlacement {
    pub container: Entity, // Which container
    pub x: u32,
    pub y: u32,
    pub rotation: Rotation, // For rotated items
}

#[derive(Component)]
pub struct Litter; // Marker for dropped/ground-loot items

#[derive(Component)]
pub struct PickupCandidate {
    pub selected: bool, 
}

// --- MESSAGE(S) --- 
#[derive(Message, Clone)]
pub struct PickupMessage {
    pub item_def: Handle<ItemDefinition>,
    pub world_entity: Entity,
}

// --- RESOURCES --- 
#[derive(Resource, Default)]
pub struct PickupArea {
    pub candidates: Vec<Entity>, 
    pub selected_index: usize, 
    pub player_pos: Vec2, 
}

#[derive(Resource)]
pub struct PlacementGhostState {
    pub active: bool, 
    pub ghost_entity: Option<Entity>,
    pub source_item: Entity, // The world item we are placing
}

// --- SYSTEMS --- 
fn spawn_bandage(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let bandage_def = asset_server.load("items/bandage.ron");

    commands.spawn((
        Name::new("Bandage"),
        Item {
            definition: bandage_def,
            quantity: 1,
        },
        Litter, 
        Transform::from_xyz(10.0, 10.0, 2.0),
        Sprite {
            custom_size: Some(Vec2::new(50.0, 50.0)), 
            image: asset_server.load("icons/bandages.png"),
            ..default()
        },
        
    ));
}

fn detect_pickup(
    player: Query<&Transform, With<Player>>, 
    litter: Query<(Entity, &GlobalTransform), With<Litter>>, 
    mut pickup_area: ResMut<PickupArea>, 
    mut commands: Commands, 
) {
    let Ok(player_transform) = player.single() else {return}; 
    pickup_area.player_pos = player_transform.translation.truncate(); 

    let mut new_candidates = Vec::new(); 
    for (entity, transform) in litter.iter() {
        let item_pos = transform.translation().truncate(); 
        if pickup_area.player_pos.distance(item_pos) < 100.0 {
            new_candidates.push(entity); 
        }
    }

    // Update if the items change 
    if new_candidates != pickup_area.candidates {
        // Clear
        for &entity in pickup_area.candidates.iter() {
            commands.entity(entity).remove::<PickupCandidate>(); 
        }

        pickup_area.candidates = new_candidates;
        pickup_area.selected_index = 0; 

        // Mark first item 
        if let Some(&entity) = pickup_area.candidates.first() {
            commands.entity(entity).insert(PickupCandidate { selected: true});  
        }
    }
}

fn cycle_pickup(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut pickup_area: ResMut<PickupArea>, 
    mut commands: Commands, 
) {
    if !keyboard.just_pressed(KeyCode::Tab) || pickup_area.candidates.is_empty() {
        return; 
    }

    if let Some(&old_entity) = pickup_area.candidates.get(pickup_area.selected_index) {
        commands.entity(old_entity).remove::<PickupCandidate>(); 
    }

    // Cycle index
    pickup_area.selected_index = (pickup_area.selected_index + 1) % pickup_area.candidates.len(); 

    // Add new selection 
    if let Some(&new_entity) = pickup_area.candidates.get(pickup_area.selected_index) {
        commands.entity(new_entity).insert(PickupCandidate { selected: true }); 
    }
}

fn confirm_pickup(
    keyboard: Res<ButtonInput<MouseButton>>, 
    pickup_area: Res<PickupArea>,
    litter: Query<&Item, With<Litter>>,
    mut messages: MessageWriter<PickupMessage>,
) {
    if !keyboard.just_pressed(MouseButton::Left) {
        return;
    }

    if let Some(&selected_entity) = pickup_area.candidates.get(pickup_area.selected_index) {
        let Ok(item) = litter.get(selected_entity) else {return}; 
        messages.write(PickupMessage {
            item_def: item.definition.clone(),
            world_entity: selected_entity,
        }); 
    }
}

// fn highlight_pickup(
//     mut candidates: Query<(&mut Sprite, &PickupCandidate), With<Litter>>, 
// ) {
//     for (mut sprite, candidate) in candidates.iter_mut() {
//         let alpha = if candidate.selected {1.0} else {0.5}; 

//         sprite.color = match sprite.color {
//             Color::Srgba {0, 1, 2, ..} => Color::srgba(r, g,  b, alpha), 
//         }
//     }
// }