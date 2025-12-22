use avian2d::{math::*, prelude::*}; 
use bevy::prelude::*; 
use crate::gameplay::player::player::Player;
use std::{collections::HashMap}; 
use serde::{Deserialize, Serialize}; 

const LOOT_SIZE: Vec2 = Vec2::new(45.0, 45.0);

pub struct ItemPlugin; 
impl Plugin for ItemPlugin {
    fn build(&self, app: &mut App){
        app
            .add_systems(Startup, (load_items, spawn_items))
            .add_systems(Update, detect_loot);
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ItemDatabase {
    pub items: Vec<ItemDefinition>, 
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ItemShape {
    pub height: i32, 
    pub width: i32, 
    pub pattern: Vec<Vec<u8>>, 
}

impl ItemShape {
    pub fn new(pattern: Vec<Vec<u8>>) -> Self {
        let height = pattern.len() as i32; 
        let width = pattern.get(0).map(|row| row.len() as i32).unwrap_or(0) as i32;

        Self {
            width, 
            height, 
            pattern,
        }
    }

    // Checks if cell is filled
    pub fn occupied(&self, x:i32, y:i32) -> bool {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return false;
        }

        self.pattern[y as usize][x as usize] != 0
    }

    pub fn rotate(&self) -> Self {
        // New dimensions: old width becomes new height, old height becomes new width
        // All indexes set to 0 again
        let mut new_pattern = vec![vec![0; self.height as usize]; self.width as usize]; 

        for y in 0..self.height {
            for x in 0..self.width {
                // Only processing occupied cells: new x rotation = height - 1 - y && new y rotation = x
                if self.pattern[y as usize][x as usize] != 0 {
                    let new_x = (self.height - 1 - y) as usize; 
                    let new_y = x as usize; 
                    new_pattern[new_y][new_x] = 1;
                }
            }
        }

        Self::new(new_pattern)
    }
}
 
#[derive(Debug, Deserialize, Serialize)]
enum ItemType {
    Consumable, 
    Essential,
    Equipment,  
}

// --- COMPONENTS --- 
#[derive(Component, Debug, Deserialize, Serialize)]
pub struct ItemDefinition {
    id: String, 
    item_type: ItemType,
    description: String, 
    max_stack: u8, // 0 - 255
    shape: ItemShape,
    rotatable: bool, 
    icon: String, 
}

// Contains necessary info for gameplay
#[derive(Clone, Component, Debug, Deserialize, Serialize)]
pub struct Item {
    pub id: String, 
    pub stack: u8, 
}

// Marker for floor items?
#[derive(Component)]
pub struct Loot; 

// Loot close enough to player for pickup
#[derive(Component)]
pub struct DetectedLoot {
    items: Vec<Entity>, 
    index: usize, 
}

// --- RESOURCES --- 
#[derive(Default, Resource)]
struct ItemRegistry {
    items: HashMap<String, ItemDefinition>, 
}

impl ItemRegistry {
    fn register(&mut self, item: ItemDefinition) {
        self.items.insert(item.id.clone(), item); 
    }

    fn get(&mut self, id: &str) -> Option<&ItemDefinition> {
        self.get(id)
    }
}

// --- SYSTEMS --- 
fn load_items(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Read the file
    let ron_str = std::fs::read_to_string("assets/items/items.ron")
        .expect("Failed to read items.ron"); 

    // Deserialize
    let database: ItemDatabase = ron::from_str(&ron_str).expect("Failed to parse items.ron"); 

    // Create registry
    let mut registry = ItemRegistry::default(); 
    for item in database.items {
        registry.register(item); 
    }

    commands.insert_resource(registry);
}

fn spawn_items(mut commands: Commands, asset_server: Res<AssetServer>) {
    let item_id = "Bandage".to_string();
for i in 0..8 {  
    commands.spawn((
        Item {id: item_id.clone(), stack: 1},
        Loot, 
        RigidBody::Static, 
        Collider::circle(LOOT_SIZE.x / 2.0),
        // Sends collision events but allows other bodies to pass through them
        Sensor,
        // Enable collision events for this entity
        CollisionEventsEnabled,
        Sprite {
            image: asset_server.load("icons/prototype_loot.png"), 
            custom_size: Some(LOOT_SIZE),
            ..default()
        }, 
        Transform::from_xyz(50.0, 50.0, 1.0),
    ));
}
}

fn detect_loot (
    mut collision_reader: MessageReader<CollisionStart>
) {
    for event in collision_reader.read() {
        // collider1 is the entity of the items
        println!("{} and {} started colliding", event.collider1, event.collider2); 
    }
}