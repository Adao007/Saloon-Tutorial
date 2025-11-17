use bevy::prelude::*; 
use crate::gameplay::inventory::items::LitterId;
use crate::gameplay::player::aim::MousePos;
use crate::gameplay::player::player::Player; 
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
            ).chain());
    }
}

// --- COMPONENTS --- 
#[derive(Component)]
pub struct Litter; // Marker for dropped/ground-loot items

#[derive(Component)]
pub struct PickupCandidate {
    pub selected: bool, 
}

#[derive(Component)]
pub struct LitterName; 

#[derive(Component)]
pub struct SelectedItemName;

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
            commands.entity(entity).insert(PickupCandidate { selected: true});  
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
        commands.entity(new_entity).insert(PickupCandidate { selected: true }); 
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