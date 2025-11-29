use std::sync::OnceLock;
use bevy::{prelude::*, sprite::{Text2dShadow}};
use bevy_common_assets::ron::RonAssetPlugin;
use crate::gameplay::player::aim::{MousePos};
use serde::{Deserialize, Serialize};
use super::pickup::{PickupPlugin, Litter}; 

pub struct ItemsPlugin; 
impl Plugin for ItemsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(RonAssetPlugin::<ItemDefinition>::new(&[".ron"]))
            .add_plugins(PickupPlugin)
            .add_systems(Startup, spawn_bandage);
    }
}

#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct ItemDefinition {
    pub name: String,
    pub item_type: ItemType,
    pub description: String,
    pub max_stack: u32,
    pub shape: Shape,
    pub rotate: bool,
    pub icon: String,

    #[serde(skip)]
    rotated_cells: OnceLock<[Vec<IVec2>; 4]>,
}

impl ItemDefinition {
    pub fn get_cells(&self, rotation: Rotation) -> &[IVec2] {
        // Compute all rotations once on first access
        self.rotated_cells.get_or_init(|| {
            [
                self.shape.to_cells(&Rotation::Zero),
                if self.rotate { self.shape.to_cells(&Rotation::Ninety) } else { vec![] },
                if self.rotate { self.shape.to_cells(&Rotation::OneEighty) } else { vec![] },
                if self.rotate { self.shape.to_cells(&Rotation::TwoSeventy) } else { vec![] },
            ]
        })
        [rotation as usize]
        .as_slice()
    }
}

// --- ENUMS ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum ItemType {
    Ammo,
    Consumable,
    Equipment,
    Salvage,
    Quest,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub enum Rotation {
    Zero,
    Ninety,
    OneEighty,
    TwoSeventy,
}

impl Rotation {
    pub fn next(&self) -> Self {
        match self {
            Rotation::Zero => Rotation::Ninety, 
            Rotation::Ninety => Rotation::OneEighty,
            Rotation::OneEighty => Rotation::TwoSeventy, 
            Rotation::TwoSeventy => Rotation::Zero,
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
pub enum Shape {
    Grid {
        width: u32,
        height: u32,
        pattern: Vec<Vec<u8>>,
    },
}

impl Shape {
     pub fn to_cells(&self, rotation: &Rotation) -> Vec<IVec2> {
        let Shape::Grid { width, height, pattern } = self;
        
        // Validate pattern matches declared dimensions
        assert_eq!(pattern.len() as u32, *height, "RON file error: pattern height mismatch");
        for (i, row) in pattern.iter().enumerate() {
            assert_eq!(row.len() as u32, *width, 
                "RON file error: pattern width mismatch on row {}", i);
        }
        
        // Collect occupied cells
        let mut cells = Vec::new();
        for (y, row) in pattern.iter().enumerate() {
            for (x, &cell) in row.iter().enumerate() {
                if cell == 1 {
                    cells.push(IVec2::new(x as i32, y as i32));
                }
            }
        }
        
        // Apply rotation
        self.rotate_cells(&cells, rotation)
    }
    
    fn rotate_cells(&self, cells: &[IVec2], rotation: &Rotation) -> Vec<IVec2> {
        let rotated: Vec<IVec2> = cells.iter().map(|pos| {
            match rotation {
                Rotation::Zero => *pos,
                Rotation::Ninety => IVec2::new(-pos.y, pos.x),
                Rotation::OneEighty => IVec2::new(-pos.x, -pos.y),
                Rotation::TwoSeventy => IVec2::new(pos.y, -pos.x),
            }
        }).collect();
        
        // Normalize so min cell is at (0,0)
        let min_x = rotated.iter().map(|p| p.x).min().unwrap_or(0);
        let min_y = rotated.iter().map(|p| p.y).min().unwrap_or(0);
        rotated.into_iter().map(|p| p - IVec2::new(min_x, min_y)).collect()
    }
}

// --- COMPONENTS --- 
#[derive(Component)]
pub struct Item {
    pub definition: Handle<ItemDefinition>,
    pub quantity: u32,
    //pub placement: Option<ItemPlacement>,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct ItemPlacement {
    pub container: Entity, // Which container
    pub x: u32,
    pub y: u32,
    pub rotation: Rotation, // For rotated items
}

#[derive(Component)]
pub struct LitterId; 

// --- SYSTEMS --- 
fn spawn_bandage(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let bandage_def = asset_server.load("items/bandage.ron");

    commands.spawn((
        Name::new("Bandage"),
        Item {
            definition: bandage_def,
            quantity: 1,
        },
        Litter, 
        Sprite {
            custom_size: Some(Vec2::new(50.0, 50.0)), 
            image: asset_server.load("icons/bandages.png"),
            ..default()
        },
        Transform::from_xyz(10.0, 150.0, 2.0),
    ));
for _i in 0..=10 {

    commands.spawn((
        Name::new("Six Shooter"),
        Item{
            definition: asset_server.load("items/six_shooter.ron"),
            quantity: 1,
        }, 
        Litter, 
        Sprite {
            custom_size: Some(Vec2::new(50.0, 50.0)), 
            image: asset_server.load("icons/six_shooter.png"),
            ..default()
        },
        Transform::from_xyz(10.0, 150.0, 2.0),
    ));
}
}

pub fn init_litter_text(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/ztn.otf"); 
    let text_font = TextFont {font: font.clone(), font_size: 25.0, ..default()}; 
    let text_justification = Justify::Center; 

    commands.spawn((
        Text2d::new("Hello"), 
        text_font.clone(), 
        TextLayout::new_with_justify(text_justification), 
        TextBackgroundColor(Color::BLACK.with_alpha(0.5)), 
        Text2dShadow::default(), 
        Transform::from_xyz(0.0, 0.0, 2.0), 
        LitterId,
    )); 
} 

pub fn move_cursor_text(
    mouse_pos: Res<MousePos>,
    mut query: Query<&mut Transform, (With<Text2d>, With<LitterId>)>, 
) {
    let offset = 10.0; 

    for mut transform in &mut query {
        transform.translation.x = mouse_pos.position.x;
        transform.translation.y = mouse_pos.position.y + offset; 
    }
}