use bevy::prelude::*; 

// Allows access to the inventory systems
#[derive(Component)]
pub struct Inventory; 

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