use bevy::prelude::*; 

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interactable{ // Marker for player interaction 
    Loot, 
}