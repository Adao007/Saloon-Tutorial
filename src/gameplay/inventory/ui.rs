use bevy::prelude::*; 

// --- COMPONENTS --- 
#[derive(Component)]
pub struct InventoryUi {
    pub activated: bool
}

#[derive(Component)]
struct MainStorage;

// --- SYSTEMS --- 
pub fn setup_ui(mut commands: Commands) {
    
    // Root node that contains all inventory parts 
    commands
        .spawn((
            Node {
                left: Val::Percent(5.0),
                top: Val::Percent(10.0),
                width: percent(60),
                height: percent(80), 
                align_self: AlignSelf::FlexStart, 
                justify_self: JustifySelf::Start,
                ..default() 
            }, 
            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.5)),
            Visibility::Hidden,
            InventoryUi {activated: false}, 

            // Other parts that make up inventory
            children![(
                Node {
                    left: Val::Percent(15.0),
                    top: Val::Percent(2.5),
                    width: percent(70),
                    height: percent(60),
                    align_self: AlignSelf::Start, 
                    justify_self: JustifySelf::Center,
                    ..default()
                },
                BackgroundColor(Color::WHITE),
                MainStorage,
            )]
    ));
}