use bevy::prelude::*; 
use std::collections::{HashMap, HashSet};
use super::items::{Item, ItemDefinition}; 

// Allows access to the inventory systems
#[derive(Component)]
pub struct Inventory{
    // Shape of Inventory
    pub valid_cells: HashSet<IVec2>, 
    // Sparse occupancy map
    pub occupied: HashMap<IVec2, Entity>, 
    // Quick list of the items for iter
    pub items: Vec<Entity>, 
}

// Define shapes for the various Inventory Entities
impl Inventory {
    // Inventory shapes 
    pub fn rectangle(width: u32, height: u32) -> Self {
        let valid_cells = (0..width as i32).flat_map(|x| (0..height as i32).map(move |y| IVec2::new(x,y))).collect(); 

        Self {
            valid_cells,
            occupied: HashMap::new(), 
            items: Vec::new(), 
        }
    }

    // Methods for Inventory
    pub fn available(&self, pos: IVec2) -> bool {
        self.valid_cells.contains(&pos) && !self.occupied.contains_key(&pos)
    }

    pub fn placeable(
        &self,
        shape: &[IVec2], 
        top_left: IVec2,
        ignore: Option<Entity>, 
    ) -> bool {
        for &cell in shape {
            let pos = top_left + cell; 
            if !self.available(pos) {
                if !self.valid_cells.contains(&pos) {
                    return false;
                }
                if let Some(occupant) = self.occupied.get(&pos) {
                    if Some(*occupant) != ignore {
                        return false; // Space is already occupied
                    }
                }
            }
        }
        true
    }

    pub fn place(&mut self, item: Entity, shape: &[IVec2], top_left: IVec2) {
        for &cell in shape {
            self.occupied.insert(top_left + cell, item); 
        }
        self.items.push(item);
    }

    pub fn remove(&mut self, item: Entity, shape: &[IVec2], top_left: IVec2) {
        for &cell in shape {
            self.occupied.remove(&(top_left + cell));
        }
        self.items.retain(|&e| e != item); 
    }

    pub fn find_valid(&self, shape: &[IVec2]) -> Option<IVec2> {
        self.valid_cells.iter()
            .find(|&&origin| self.placeable(shape, origin, None))
            .copied()
    }
}

#[derive(Event)]
pub struct PickupItem {
    pub item_def: Handle<ItemDefinition>,
    pub world_entity: Entity, 
}

#[derive(Event)]
pub struct InventoryAction {
    pub item: Entity,
    pub new_position: IVec2,
    pub new_rotation: u8, 
}

#[derive(Resource)]
pub struct InventoryUIState{
    pub is_open: bool, 
    pub cursor_world_pos: Vec2, 
}

#[derive(Component)]
pub struct PlacementGhost; 

#[derive(Component)]
pub struct Hotbar; 

// TODO! 
// Open Inventory
// Changes based on what is equipped? 

// Hotbar Inventory --> Equipment/Consumables
// Holsters: Can store weapons, has access to hotbar
// Pockets, Utility Belts, etc: Can store Consumables and Salvage, has access to hotbar

// Main Storage --> Bags, Backpacks, etc
// Can store any items, does not affect hotbar. 

// Items are stored when they fit into place. 
// Items can be rotated
// Items can be inspected? 
// Change Hotbar order: 1, 2, 3, 4, etc 

#[derive(Resource)]
pub struct PlayerInventory {
    pub entity: Entity, 
}

fn setup_inventory(mut commands: Commands) {
    let inventory = commands.spawn((
        Inventory::rectangle(8, 10),
        Name::new("Player Inventory"),
    )).id(); 

    commands.insert_resource(PlayerInventory { entity: inventory }); 

    commands.insert_resource(InventoryUIState {
        is_open: false, 
        cursor_world_pos: Vec2::ZERO,
    });
}