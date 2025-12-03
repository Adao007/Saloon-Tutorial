use bevy::prelude::*; 
use crate::gameplay::inventory::pickup::GhostItem;
use crate::gameplay::player::aim::MousePos;
use crate::gameplay::inventory::inventory::PlayerInventory;
use crate::gameplay::inventory::inventory::Inventory;
use crate::gameplay::inventory::items::ItemDefinition;
use crate::gameplay::inventory::items::ItemPlacement;
use crate::gameplay::inventory::items::Item;
use crate::gameplay::inventory::pickup::PlacementGhostState;
use crate::gameplay::inventory::pickup::PlacementGhost;

// --- COMPONENTS --- 
#[derive(Component)]
pub struct ItemDrag {
    pub item: Entity, 
    pub source_slot: IVec2, 
}

#[derive(Component)]
pub struct HoverInventory; 

// --- EVENTS --- 
#[derive(Event)]
pub struct ClickInventory {
    pub cell: IVec2, 
    pub button: MouseButton,
}

// --- SYSTEMS --- 
pub fn detect_inventory_click(
    asset_server: Res<AssetServer>, 
    cursor: Res<MousePos>, 
    inventories: Query<&Inventory>, 
    items: Query<(Entity, &ItemPlacement, &Item)>, 
    item_definitions: Res<Assets<ItemDefinition>>,
    mouse: Res<ButtonInput<MouseButton>>,
    player_inventory: Res<PlayerInventory>, 
    mut commands: Commands, 
    mut ghost_state: ResMut<PlacementGhostState>,
) {
    if !mouse.just_pressed(MouseButton::Left) { return; }
    let Ok(inventory) = inventories.get(player_inventory.entity) else { return };

    // Get clicked cell in world coords to inventory grid
    let grid_position = (cursor.position / 32.0).floor().as_ivec2(); // Why? 

    // Check for item
    if let Some((item_entity, placement, item)) = items.iter().find(|(_, p, _)| p.x as i32 == grid_position.x && p.y as i32 == grid_position.y) {
        let Some(def) = item_definitions.get(&item.definition) else { return; };
        
        commands.entity(item_entity).remove::<ItemPlacement>(); // Remove from the inventory while dragging 
        let ghost = commands.spawn((
            PlacementGhost,
            ItemDrag { item: item_entity, source_slot: grid_position},
            GhostItem { def: item.definition.clone(), rotation: placement.rotation}, 
            Sprite::from_image(asset_server.load(&def.icon)), 
            Transform::from_xyz(grid_position.x as f32 * 32.0, grid_position.y as f32 * 32.0, 10.0),
        )).id(); 

        ghost_state.active = true; 
        ghost_state.ghost_entity = Some(ghost); 
        ghost_state.source_item = item_entity; 
    }
}

pub fn inventory_drag(
    mouse: Res<ButtonInput<MouseButton>>,
    dragged: Query<&ItemDrag>, 
    ghost: Query<&GhostItem>, // Contains our definition and rotation 
    ghost_transform: Query<&Transform, With<PlacementGhost>>, 
    items: Query<&Item>, 
    item_definitions: Res<Assets<ItemDefinition>>,
    mut inventories: Query<&mut Inventory>, 
    mut state: ResMut<PlacementGhostState>, 
    mut commands: Commands, 
) { 
    if !state.active || !mouse.just_pressed(MouseButton::Left) {return;}
    let Some(ghost_entity) = state.ghost_entity else {return}; 
    let Ok(ghost_data) = ghost.get(ghost_entity) else {return};
    let Ok(transform) = ghost_transform.get(ghost_entity) else {return;}; 
    let Ok(dragged_info) = dragged.get(ghost_entity) else {return}; 
    let Ok(item_component) = items.get(dragged_info.item) else {return};
    let Some(def) = item_definitions.get(&item_component.definition) else {return}; 
    let Ok(mut inventory) = inventories.get_mut(state.target_inventory) else { return }; 
    let target_pos = (transform.translation.truncate() / 32.0).floor().as_ivec2(); 
    let cells = def.get_cells(ghost_data.rotation); 

    // Validate placement
    if inventory.placeable(&cells, target_pos, Some(dragged_info.item)) {
        commands.entity(dragged_info.item).insert(ItemPlacement {
            container: state.target_inventory,
            x: target_pos.x as u32, 
            y: target_pos.y as u32,
            rotation: ghost_data.rotation,
        });

        inventory.place(dragged_info.item, &cells, target_pos); 

        // Cleanup 
        commands.entity(ghost_entity).despawn(); 
        state.active = false; 
    }
}


// --- DEBUG SYSTEMS --- 
pub fn debug_mouse_clicks(
    mouse: Res<ButtonInput<MouseButton>>, 
    mouse_pos: Res<MousePos>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        println!("Mouse clicked at {:?}", mouse_pos.position);
    }
}