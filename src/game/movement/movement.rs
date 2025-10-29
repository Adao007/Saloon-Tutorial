use bevy::prelude::*; 
use crate::player::setup::Player; 

pub struct Movement; 
impl Plugin for Movement {
    fn build(&self, app: &mut App) {

    }
}

fn walk(
    player_query: Query<&mut Transform, With<Player>>
) {

}