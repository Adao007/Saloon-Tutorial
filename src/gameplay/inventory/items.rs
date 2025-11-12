use bevy::{prelude::*};
use bevy_common_assets::ron::RonAssetPlugin;
use crate::gameplay::player::{
    aim::{MousePos, get_mouse_position},
    player::Player,
};
    use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct ItemsPlugin; 
impl Plugin for ItemsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(RonAssetPlugin::<ItemDefinition>::new(&[".ron"]))
            .add_systems(Startup, spawn_bandage)
            .add_systems(Update, (pickup_input.after(get_mouse_position), handle_pickup));
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum ItemType {
    Ammo,
    Consumable,
    Equipment,
    Salvage,
    Quest,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct ItemPlacement {
    pub container: Entity, // Which container
    pub x: u32,
    pub y: u32,
    pub rotation: Rotation, // For rotated items
}

#[derive(Component)]
pub struct Item {
    pub definition: Handle<ItemDefinition>,
    pub quantity: u32,
    //pub placement: Option<ItemPlacement>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub enum Rotation {
    Zero,
    Ninety,
    OneEighty,
    TwoSeventy,
}

#[derive(Deserialize, Clone, Debug)]
pub enum Shape {
    Grid {
        width: u32,
        height: u32,
        pattern: Vec<Vec<u8>>,
    },
}

// Marker for dropped/ground-loot items
#[derive(Component)]
pub struct Litter; 

#[derive(Component)]
pub struct Container {
    pub width: u32,
    pub height: u32,
    pub grid: HashMap<(u32, u32), Entity>,
    pub items: Vec<Entity>,
}

impl Container {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            grid: HashMap::new(),
            items: Vec::new(),
        }
    }

    pub fn is_cell_free(&self, x: u32, y: u32) -> bool {
        x < self.width && y < self.height && !self.grid.contains_key(&(x, y))
    }
}

#[derive(Message, Clone, Copy)]
pub struct PickUpMessage {
    pub item: Entity,
    pub by: Entity, 
}

// Systems for items
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

fn pickup_input(
    mouse: Res<ButtonInput<MouseButton>>, 
    cursor: Res<MousePos>,
    player: Query<(Entity, &Transform), With<Player>>,
    pickup_items: Query<(Entity, &GlobalTransform), With<Litter>>, 
    mut commands: Commands,
) {
    if mouse.just_pressed(MouseButton::Left) {
        let Ok((player, player_tf)) = player.single() else { return }; 
        let player_pos = player_tf.translation.truncate(); // 2-D
        
        if cursor.position.distance(player_pos) > 100.0 {
            return;
        }

        for (pickup, transform) in &pickup_items {
            let item_pos = transform.translation().truncate(); 
            if cursor.position.distance(item_pos) < 100.0 {
                commands.entity(pickup).remove::<Litter>(); 
                commands.write_message(PickUpMessage{
                    item: pickup, 
                    by: player,
                });
                break; 
            }
        }
    }
}

fn handle_pickup(
    mut events: MessageReader<PickUpMessage>, 
    mut commands: Commands,
) {
    for event in events.read() {
        info!("{:?} picked up {:?}", event.by, event.item);
        println!("Picked up item!!!");
    }
}