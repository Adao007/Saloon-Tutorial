use bevy::prelude::*;
use serde::{Deserialize, Serialize}; 
use crate::gameplay::inventory::ui::setup_ui;
use crate::gameplay::inventory::ui::InventoryUi;
use crate::gameplay::item::items::Item;
use crate::gameplay::player::player::Player;

pub struct InventoryPlugin;
impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app 
            .add_systems(Startup, setup_ui)
            .add_systems(Update, activate_player_inventory);
    }
}

// --- COMPONENTS --- 
#[derive(Component, Default, Serialize, Deserialize, Clone, Debug)]
pub struct Inventory {
    pub items: Vec<Item>, 
    pub searching: bool
}

// TODO: support for quantity mutations as well. 
impl Inventory {
    fn add(&mut self, item: Item) {
        self.items.push(item);
    }

    fn remove(&mut self, item: Item) {
        let item_remove = item.id; 
        if let Some(pos) = self.items.iter().position(|x| x.id == item_remove) {
            self.items.remove(pos); 
        }
    }    
}

#[derive(Component)]
pub struct Searching; 

// --- SYSTEMS --- 
fn activate_player_inventory(
    keyboard_input: Res<ButtonInput<KeyCode>>, 
    player: Single<Entity, With<Player>>,
    mut commands: Commands,
    mut inventory: Single<&mut Inventory, With<Player>>, 
    mut visibility_query: Query<(&mut Visibility, &mut InventoryUi)>,
) {
    if !keyboard_input.just_pressed(KeyCode::KeyI) {
        return; 
    }

    for (mut visibility, mut ui) in visibility_query.iter_mut() {
        visibility.toggle_visible_hidden();
        ui.activated = !ui.activated;
        inventory.searching = !inventory.searching; 
        
        if inventory.searching {
            commands.entity(*player).insert(Searching); 
        }
        else {
            commands.entity(*player).remove::<Searching>(); 
        }
    }
}