use bevy::prelude::*; 
use std::collections::HashMap; 
use serde::{Deserialize, Serialize}; 

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum ItemType {
    Ammo,
    Consumable,
    Equipment, 
    Salvage, 
    Quest, 
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

#[derive(Clone, Copy, Debug)]
pub struct ItemPlacement {
    pub container: Entity, // Which container
    pub x: u32, 
    pub y: u32, 
    pub rotation: Rotation, // For rotated items
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub enum Rotation {
    Zero, 
    Ninety,
    OneEighty,
    TwoSeventy,
}

#[derive(Component)]
pub struct Item {
    pub definition: Handle<ItemDefinition>, 
    pub quantity: u32,
    pub placement: Option<ItemPlacement>, 
}

#[derive(Deserialize, Clone, Debug)]
pub enum Shape {
    Grid {
        width: u32, 
        height: u32, 
        pattern: Vec<Vec<u8>>,
    },
}

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
        x < self.width && y < self.height && !self.grid.contains_key(&(x,y))
    }
}