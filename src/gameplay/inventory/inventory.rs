use bevy::prelude::*; 
use crate::gameplay::inventory::ui::setup_ui;
use crate::gameplay::inventory::ui::InventoryUi;

pub struct InventoryPlugin;
impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app 
            .add_systems(Startup, setup_ui)
            .add_systems(Update, activate_inventory);
    }
}

// --- SYSTEMS --- 
fn activate_inventory(
    keyboard_input: Res<ButtonInput<KeyCode>>, 
    mut visibility_query: Query<(&mut Visibility, &mut InventoryUi)>,
) {
    if !keyboard_input.just_pressed(KeyCode::KeyI) {
        return; 
    }

    for (mut visibility, mut ui) in visibility_query.iter_mut() {
        visibility.toggle_visible_hidden();
        ui.activated = !ui.activated;
    }
}