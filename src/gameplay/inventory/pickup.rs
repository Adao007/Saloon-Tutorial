use bevy::prelude::*; 
use crate::gameplay::inventory::items::LitterId;
use crate::gameplay::player::aim::MousePos;
use crate::gameplay::player::player::Player; 
use crate::gameplay::inventory::items::Rotation;
use crate::gameplay::inventory::items::ItemPlacement;
use crate::gameplay::inventory::inventory::PlayerInventory;
use crate::gameplay::inventory::inventory::Inventory;
use super::items::{Item, ItemDefinition}; 

pub struct PickupPlugin; 
impl Plugin for PickupPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<PickupArea>()
            .init_resource::<PlacementGhostState>()
            .add_message::<PickupMessage>()
            .add_systems(Update, (
                detect_pickup,
                cycle_pickup, 
                confirm_pickup,
                display_item_name,
                handle_pickup_message, 
                update_ghost_placement, 
                cycle_ghost_rotation,
                finalize_ghost_placement,
                cancel_ghost_placement,
            ).chain());
    }
}

// --- COMPONENTS --- 
#[derive(Component)]
pub struct Litter; // Marker for dropped/ground-loot items

#[derive(Component)]
pub struct PickupCandidate {
}

#[derive(Component)]
pub struct LitterName; 

#[derive(Component)]
pub struct SelectedItemName;

#[derive(Component)]
pub struct GhostItem {
    pub def: Handle<ItemDefinition>, 
    pub rotation: Rotation, 
}

#[derive(Component)]
pub struct PlacementGhost; 

// --- RESOURCES --- 
#[derive(Resource, Default)]
pub struct PickupArea {
    pub candidates: Vec<Entity>, 
    pub selected_index: usize, 
    pub player_pos: Vec2, 
}

#[derive(Resource)]
pub struct PlacementGhostState {
    pub active: bool, 
    pub ghost_entity: Option<Entity>,
    pub source_item: Entity, // The world item we are placing
    pub target_inventory: Entity,
}

impl Default for PlacementGhostState {
    fn default() -> Self {
        Self {
            active: false,
            ghost_entity: None,
            source_item: Entity::PLACEHOLDER,
            target_inventory: Entity::PLACEHOLDER,
        }
    }
}

// --- MESSAGE(S) --- 
#[derive(Message, Clone)]
pub struct PickupMessage {
    pub item_def: Handle<ItemDefinition>,
    pub world_entity: Entity,
}

// ---SYSTEMS --- 
fn detect_pickup(
    player: Query<&Transform, With<Player>>, 
    litter: Query<(Entity, &GlobalTransform), With<Litter>>, 
    mut pickup_area: ResMut<PickupArea>, 
    mut commands: Commands, 
) {
    let Ok(player_transform) = player.single() else {return}; 
    pickup_area.player_pos = player_transform.translation.truncate(); 

    let mut new_candidates = Vec::new(); 
    for (entity, transform) in litter.iter() {
        let item_pos = transform.translation().truncate(); 
        if pickup_area.player_pos.distance(item_pos) < 100.0 {
            new_candidates.push(entity); 
        }
    }

    new_candidates.sort(); 

    // Update if the items change 
    if new_candidates != pickup_area.candidates {
        println!("Found {} items in pickup range", new_candidates.len());
        // Clear
        for &entity in pickup_area.candidates.iter() {
            commands.entity(entity).remove::<PickupCandidate>(); 
        }

        pickup_area.candidates = new_candidates;
        pickup_area.selected_index = 0; 

        // Mark first item 
        if let Some(&entity) = pickup_area.candidates.first() {
            commands.entity(entity).insert(PickupCandidate{});  
        }
    }
}

fn cycle_pickup(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut pickup_area: ResMut<PickupArea>, 
    mut commands: Commands, 
) {
    if !keyboard.just_pressed(KeyCode::Tab) || pickup_area.candidates.is_empty() {
        return; 
    }

    println!("Selected item index: {}", pickup_area.selected_index);

    if let Some(&old_entity) = pickup_area.candidates.get(pickup_area.selected_index) {
        commands.entity(old_entity).remove::<PickupCandidate>(); 
    }

    // Cycle index
    pickup_area.selected_index = (pickup_area.selected_index + 1) % pickup_area.candidates.len(); 

    // Add new selection 
    if let Some(&new_entity) = pickup_area.candidates.get(pickup_area.selected_index) {
        commands.entity(new_entity).insert(PickupCandidate{}); 
    }
}

fn confirm_pickup(
    keyboard: Res<ButtonInput<MouseButton>>, 
    pickup_area: Res<PickupArea>,
    litter: Query<&Item, With<Litter>>,
    mut messages: MessageWriter<PickupMessage>,
) {
    if !keyboard.just_pressed(MouseButton::Left) {
        return;
    }

    if let Some(&selected_entity) = pickup_area.candidates.get(pickup_area.selected_index) {
        let Ok(item) = litter.get(selected_entity) else {return}; 
         println!("Attempting to pickup item: {:?}", selected_entity);
        messages.write(PickupMessage {
            item_def: item.definition.clone(),
            world_entity: selected_entity,
        }); 
    }
}

fn display_item_name(
    selected_item: Query<(&Item, &GlobalTransform, &Sprite), (With<PickupCandidate>, With<Litter>)>,
    item_defs: Res<Assets<ItemDefinition>>,
    mouse_pos: Res<MousePos>, 
    litter_id: Single<Entity, (With<Text2d>, With<LitterId>)>, 
    mut writer: Text2dWriter, 
) {
    let Ok((item, transform, sprite)) = selected_item.single() else {
        *writer.text(*litter_id, 0) = "".to_string(); 
        return; 
    }; 

    let cursor_pos = mouse_pos.position; 
    let item_pos = transform.translation().truncate(); 
    let half_size = sprite.custom_size.unwrap_or(Vec2::splat(32.0)) / 2.0; 

    let is_hovering = cursor_pos.x >= item_pos.x - half_size.x 
        && cursor_pos.x <= item_pos.x + half_size.x
        && cursor_pos.y >= item_pos.y - half_size.y
        && cursor_pos.y <= item_pos.y + half_size.y;

    if is_hovering {
        if let Some(def) = item_defs.get(&item.definition) {
            *writer.text(*litter_id, 0) = def.name.clone(); 
        }
    }
    else {
        *writer.text(*litter_id, 0) = "".to_string(); 
    }
}

pub fn handle_pickup_message(
    mut messages: MessageReader<PickupMessage>, 
    mut commands: Commands,
    mut inventories: Query<&mut Inventory>,
    mut ghost_state: ResMut<PlacementGhostState>, 
    asset_server: Res<AssetServer>,
    item_defs: Res<Assets<ItemDefinition>>, 
    player_inventory: Res<PlayerInventory>, 
) {
    for message in messages.read() {
        let Some(def) = item_defs.get(&message.item_def) else { continue }; 
        let Ok(mut inventory) = inventories.get_mut(player_inventory.entity) else { continue }; 
        let cells = def.get_cells(Rotation::Zero); 

        // Auto-Place
        if let Some(pos) = inventory.find_valid(&cells) {
            info!("Auto-placed {} at {:?}", def.name, pos); 

            let item_entity = commands.spawn((
                Item {
                    definition: message.item_def.clone(),
                    quantity: 1, 
                }, 
                ItemPlacement {
                    container: player_inventory.entity, 
                    x: pos.x as u32,
                    y: pos.y as u32, 
                    rotation: Rotation::Zero, 
                }, 
            )).id(); 

            inventory.place(item_entity, &cells, pos);
            commands.entity(message.world_entity).despawn();
            continue;
        }

        // MANUAL PLACEMENT
        info!("Manual placement for {}", def.name);
        commands.entity(message.world_entity).insert(Visibility::Hidden);
        
        let ghost = commands.spawn((
            PlacementGhost,
            GhostItem {
                def: message.item_def.clone(),
                rotation: Rotation::Zero,
            },
            Sprite {
                image: asset_server.load(&def.icon), // Use icon for ghost
                color: Color::srgba(1.0, 1.0, 1.0, 0.5),
                custom_size: Some(Vec2::new(32.0, 32.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 10.0),
        )).id();
        
        *ghost_state = PlacementGhostState {
            active: true,
            ghost_entity: Some(ghost),
            source_item: message.world_entity,
            target_inventory: player_inventory.entity,
        };
    }
}

fn update_ghost_placement(
    state: Res<PlacementGhostState>,
    mut ghost_q: Query<(&GhostItem, &mut Transform, &mut Sprite)>,
    mouse_pos: Res<MousePos>,
    inventories: Query<&Inventory>,
    item_defs: Res<Assets<ItemDefinition>>,
) {
    if !state.active || state.ghost_entity.is_none() { return };
    
    let Ok((ghost, mut transform, mut sprite)) = ghost_q.get_mut(state.ghost_entity.unwrap()) else { return };
    let Ok(inventory) = inventories.get(state.target_inventory) else { return };
    let Some(def) = item_defs.get(&ghost.def) else { return };
    
    // Snap to grid
    let grid_pos = (mouse_pos.position / 32.0).floor().as_ivec2();
    transform.translation = (grid_pos.as_vec2() * 32.0).extend(10.0);
    
    // Validate placement
    let cells = def.get_cells(ghost.rotation);
    if inventory.placeable(&cells, grid_pos, None) {
        sprite.color.set_alpha(0.5); // Green (keep original color, just opacity)
    } else {
        sprite.color.set_alpha(0.2); // Red (translucent red)
    }
}

fn cycle_ghost_rotation(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<PlacementGhostState>,
    mut ghost_q: Query<&mut GhostItem>,
    item_defs: Res<Assets<ItemDefinition>>,
) {
    if !keyboard.just_pressed(KeyCode::KeyR) || !state.active { return };
    
    let Some(ghost_entity) = state.ghost_entity else { return };
    let Ok(mut ghost) = ghost_q.get_mut(ghost_entity) else { return };
    let Some(def) = item_defs.get(&ghost.def) else { return };
    
    if def.rotate {
        ghost.rotation = ghost.rotation.next();
    }
}

fn finalize_ghost_placement(
    mouse: Res<ButtonInput<MouseButton>>,
    mut state: ResMut<PlacementGhostState>,
    ghost_q: Query<(&GhostItem, &Transform)>,
    mut inventories: Query<&mut Inventory>,
    item_defs: Res<Assets<ItemDefinition>>,
    mut commands: Commands,
) {
    if !state.active || !mouse.just_pressed(MouseButton::Left) { return };
    
    let Some(ghost_entity) = state.ghost_entity else { return };
    let Ok((ghost, transform)) = ghost_q.get(ghost_entity) else { return };
    let Ok(mut inventory) = inventories.get_mut(state.target_inventory) else { return };
    let Some(def) = item_defs.get(&ghost.def) else { return };
    
    let grid_pos = (transform.translation.truncate() / 32.0).floor().as_ivec2();
    let cells = def.get_cells(ghost.rotation);
    
    if inventory.placeable(&cells, grid_pos, None) {
        let item_entity = commands.spawn((
            Item {
                definition: ghost.def.clone(),
                quantity: 1,
            },
            ItemPlacement {
                container: state.target_inventory,
                x: grid_pos.x as u32,
                y: grid_pos.y as u32,
                rotation: ghost.rotation,
            },
        )).id();
        
        inventory.place(item_entity, &cells, grid_pos);
        commands.entity(state.source_item).despawn();
        commands.entity(ghost_entity).despawn();
        state.active = false;
    }
}

fn cancel_ghost_placement(
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<PlacementGhostState>,
    mut commands: Commands,
) {
    if !state.active { return };
    
    let should_cancel = mouse.just_pressed(MouseButton::Right) 
        || keyboard.just_pressed(KeyCode::Escape);
    
    if should_cancel {
        commands.entity(state.source_item).remove::<Visibility>(); // Show world item again
        if let Some(ghost) = state.ghost_entity {
            commands.entity(ghost).despawn();
        }
        state.active = false;
        state.ghost_entity = None;
    }
}