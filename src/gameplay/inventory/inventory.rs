use bevy::{prelude::*}; 
use crate::gameplay::inventory::interaction::*;
use crate::gameplay::inventory::items::{ItemsPlugin, Item, ItemPlacement};
use crate::gameplay::inventory::pickup::handle_pickup_message;
use crate::gameplay::player::player::Player;
use std::collections::{HashMap, HashSet};
use super::items::{ItemDefinition}; 

pub struct InventoryPlugin; 
impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<InventoryUIState>()
            .add_systems(Startup, (
                setup_inventory, 
                //setup_minimal_ui,
                setup_inventory_ui,
                debug_ui_exists,
            ).chain())
            .add_systems(Update, (
                toggle_inventory_ui,
                update_inventory_visibility,
                // GIZMOS: visualize_inventory_grid,
                sync_inventory_ui.after(handle_pickup_message),
                detect_inventory_click,
                inventory_drag,
                debug_mouse_clicks,
            ).chain())
            .add_plugins(ItemsPlugin);
    }
}

// --- COMPONENTS ---- 
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
                    return false; // Item is sticking out of the "Inventory" container
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

#[derive(Component)]
pub struct InventoryUI; 

#[derive(Component)]
pub struct Hotbar; 

#[derive(Component)]
pub struct CellPosition(pub IVec2);

// --- EVENTS --- 
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

// --- RESOURCES --- 
#[derive(Resource, Default)]
pub struct InventoryUIState{
    pub is_open: bool, 
    pub cursor_world_pos: Vec2, 
}

#[derive(Resource)]
pub struct PlayerInventory {
    pub entity: Entity, 
}

// --- SYSTEMS --- 
pub fn setup_inventory(mut commands: Commands) {
    let inventory = commands.spawn((
        Inventory::rectangle(4, 4),
        Name::new("Player Inventory"),
    )).id(); 

    commands.insert_resource(PlayerInventory { entity: inventory }); 

    commands.insert_resource(InventoryUIState {
        is_open: false, 
        cursor_world_pos: Vec2::ZERO,
    });
}

pub fn toggle_inventory_ui(
    keyboard: Res<ButtonInput<KeyCode>>, 
    mut ui_state: ResMut<InventoryUIState>, 
) {
    if keyboard.just_pressed(KeyCode::KeyI) {
        ui_state.is_open = !ui_state.is_open; 
        println!("Inventory UI: {}", ui_state.is_open); 
    }
}

pub fn update_inventory_visibility(
    ui_state: Res<InventoryUIState>, 
    mut ui_root: Query<&mut Visibility, With<InventoryUI>>, 
) {
    let Ok(mut visibility) = ui_root.single_mut() else {
        println!("Warning: No InventoryUI entity found"); 
        return; 
    }; 

    *visibility = if ui_state.is_open {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
}

pub fn visualize_inventory_grid(
    ui_state: Res<InventoryUIState>,
    player_inv: Res<PlayerInventory>,
    inventories: Query<&Inventory>,
    player: Query<&Transform, With<Player>>, // ADD THIS
    mut gizmos: Gizmos,
) {
    if !ui_state.is_open { return };
    
    let Ok(inventory) = inventories.get(player_inv.entity) else { return };
    let Ok(player_tf) = player.single() else { return };
    let player_pos = player_tf.translation.truncate(); // Player's world position
    
    // Draw grid offset by player position so it follows them
    for &cell in &inventory.valid_cells {
        let world_pos = (player_pos + cell.as_vec2() * 32.0).extend(10.0);
        gizmos.rect(world_pos, Vec2::splat(32.0), Color::srgb(0.0, 1.0, 0.0));
    }
    
    // Draw occupied cells
    for (&pos, _) in &inventory.occupied {
        let world_pos = (player_pos + pos.as_vec2() * 32.0).extend(10.0);
        gizmos.rect(world_pos, Vec2::splat(32.0), Color::srgb(1.0, 0.0, 0.0));
    }
}

fn setup_inventory_ui(
    player_inv: Res<PlayerInventory>,
    inventories: Query<&Inventory>,
    mut commands: Commands,
) {
    let Ok(inventory) = inventories.get(player_inv.entity) else { return };
    
    // Spawn root container (invisible, just a parent)
    let root = commands.spawn((
        InventoryUI,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(50.0),
            top: Val::Percent(50.0),
            ..default()
        },
        BackgroundColor(Color::srgb(1.0, 0.0, 1.0)),
        Transform::from_translation(Vec3::ZERO),
        Visibility::Hidden,
    )).id();
    
    // Spawn cells ONLY for valid positions
    for &pos in &inventory.valid_cells {
        let cell = commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(pos.x as f32 * 32.0),
                top: Val::Px(pos.y as f32 * 32.0),
                width: Val::Px(32.0),
                height: Val::Px(32.0),
                border: UiRect::all(Val::Px(4.0)),
                ..default()
            },
            BorderColor::all(Color::srgb(0.5, 0.5, 0.5)), // 30% gray border
            CellPosition(pos), // Marker to identify which cell this is
        )).id();
        
        commands.entity(root).add_child(cell);
    }
}

fn debug_ui_exists(
    ui_root: Query<Entity, With<InventoryUI>>,
    cells: Query<&CellPosition>,
) {
    if let Ok(root) = ui_root.single() {
        println!("UI root spawned: {:?}", root);
        println!("Grid cells found: {}", cells.iter().count());
    } else {
        println!("ERROR: No UI root found!");
    }
}

fn sync_inventory_ui (
    ui_state: Res<InventoryUIState>, 
    player_inventory: Res<PlayerInventory>, 
    inventories: Query<&Inventory>, 
    items: Query<(&Item, &ItemPlacement)>, 
    item_defs: Res<Assets<ItemDefinition>>, 
    asset_server: Res<AssetServer>, 
    mut commands: Commands,
    mut cells: Query<(Entity, &CellPosition, &mut BackgroundColor, &mut BorderColor, Option<&mut ImageNode>)>, 
) {
    if !ui_state.is_open {
        return
    }; 

    let Ok(inventory) = inventories.get(player_inventory.entity) else { return }; 

    // Mark occupied cells and add item icons
    for (cell_entity, cell_pos, mut bg_color, mut border_color, image_node) in cells.iter_mut() {
        *bg_color = Color::NONE.into();
        *border_color = Color::srgba(0.0, 1.0, 0.0, 0.0).into(); 
        
        if let Some(&item_entity) = inventory.occupied.get(&cell_pos.0) {
            *bg_color = Color::srgb(0.3, 0.3, 0.3).into(); 
            *border_color = Color::srgb(1.0, 0.0, 0.0).into(); 

            // Add or update icons 
            if let Ok((item, _)) = items.get(item_entity) {
                if let Some(def) = item_defs.get(&item.definition) {
                    if let Some(mut node) = image_node { 
                        node.image = asset_server.load(&def.icon);
                    }
                    else {
                        commands.entity(cell_entity).insert(ImageNode {
                            image: asset_server.load(&def.icon),
                            color: Color::WHITE,
                            flip_x: false,
                            flip_y: false, 
                            ..default()
                        });                         
                    }
                }
            }
        }
    }
}