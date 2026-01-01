use bevy::prelude::*; 
use crate::gameplay::player::aim::{get_mouse_position, MousePos};

const FONT_SIZE: f32 = 15.0;
const OFFSET_X: f32 = 0.0; 
const OFFSET_Y: f32 = 0.0; 

pub struct CursorPlugin; 
impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_cursor_ui)
            .add_systems(Update, (
                place_ui,
                set_cursor_info,
            )); 
    }
}

#[derive(Component)]
struct CursorUi;

#[derive(Component)]
struct CursorText; // Marker for Text info near cursor

fn setup_cursor_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    let font_handle: Handle<Font> = asset_server.load("fonts/ztn.otf");

    commands
        .spawn((
            CursorUi, 
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                ..default()
            }, 
            BackgroundColor(Color::srgba(0.10, 0.10, 0.10, 0.5)), 
            Visibility::Visible,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    CursorText,
                    Text::new("War, War never changes."),
                    TextFont {
                        font: font_handle.clone(),
                        font_size: FONT_SIZE,
                        ..default()
                    }, 
                    TextColor(Color::srgb(1.0, 1.0, 1.0)),
                ));
        });
}

fn place_ui(
    windows: Query<&Window>,
    mut ui_query: Query<&mut Node, With<CursorUi>>, 
) {
    match windows.single() {
        Ok(window) => {
            if let Some(cursor_pos) = window.cursor_position() {
                for mut ui in &mut ui_query {
                    ui.left = Val::Px(cursor_pos.x - OFFSET_X);
                    ui.top = Val::Px(cursor_pos.y - OFFSET_Y); 
                }
            }
        }, 
        Err(_) => { println!("Error: Window data not found.")}
    } 
}

fn set_cursor_info(
    mut ui_query: Query<&mut Text, With<CursorText>>, 
) {
    for mut text in &mut ui_query {
        text.0 = "Hey, it changed!".to_string(); 
    }
}