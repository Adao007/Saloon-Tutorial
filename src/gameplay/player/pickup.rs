use bevy::prelude::*; 
use crate::gameplay::item::items::*; 
use crate::gameplay::player::player::Player;

// --- COMPONENTS --- 

// --- RESOURCES ---

// --- SYSTEMS ---
pub fn detect_loot(
    loot_query: Query<&Transform, With<Loot>>, 
    player_query: Single<&Transform, With<Player>>, 
) {
    
}