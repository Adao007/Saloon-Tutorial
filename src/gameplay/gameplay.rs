use crate::gameplay::{
    inventory::inventory::InventoryPlugin,
    item::items::ItemPlugin,
    player::player::PlayerPlugin, 
    stage::stage::StagePlugin,
};
use bevy::prelude::*;

pub struct GameplayPlugin;
impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((
                StagePlugin, 
                PlayerPlugin, 
                ItemPlugin,
                InventoryPlugin,
            )); 
    }
}
