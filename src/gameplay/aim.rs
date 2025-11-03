use super::player::Player;
use bevy::prelude::*;

#[derive(Resource)]
pub struct MousePos {
    pub position: Vec2,
}

#[derive(Component)]
pub struct VisibilityCone {
    pub range: f32,
    pub angle: f32,
    pub direction: Vec2,
}

pub fn get_mouse_position(
    q_window: Query<&Window>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut mouse_pos: ResMut<MousePos>,
) {
    let (camera, camera_transform) = q_camera.single().unwrap();
    let window = q_window.single().unwrap();

    if let Some(world_pos) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
        .map(|ray| ray.origin.truncate())
    {
        mouse_pos.position = world_pos;
    }
}

pub fn snap_aim(mut player_query: Query<&mut Transform, With<Player>>, mouse_pos: Res<MousePos>) {
    let aim_translation = mouse_pos.position.xy();
    for mut player_transform in &mut player_query {
        let to_aim = (aim_translation - player_transform.translation.xy()).normalize();
        let rotate = Quat::from_rotation_arc(Vec3::Y, to_aim.extend(0.));
        player_transform.rotation = rotate;
    }
}

pub fn rotate_aim(
    mut player_query: Query<&mut Transform, With<Player>>,
    mouse_pos: Res<MousePos>,
    time: Res<Time>,
) {
    let rotation_speed = f32::to_radians(360.0);
    let aim_translation = mouse_pos.position.xy();
    for mut player_transform in &mut player_query {
        let player_forward = (player_transform.rotation * Vec3::Y).xy();
        let to_aim = (aim_translation - player_transform.translation.xy()).normalize();
        let forward_dot = player_forward.dot(to_aim);
        if (forward_dot - 1.0).abs() < f32::EPSILON {
            continue;
        }

        let player_right = (player_transform.rotation * Vec3::X).xy();
        let right_dot = player_right.dot(to_aim);
        let rotation_sign = -f32::copysign(1.0, right_dot);
        let max_angle = ops::acos(forward_dot.clamp(-1.0, 1.0));

        let rotation_angle = rotation_sign * (rotation_speed * time.delta_secs()).min(max_angle);
        player_transform.rotate_z(rotation_angle);
    }
}

pub fn ray_segment_interaction(
    ray_origin: Vec2,
    ray_dir: Vec2,
    seg_start: Vec2,
    seg_end: Vec2,
) -> Option<Vec2> {
    let v1 = ray_origin - seg_start;
    let v2 = seg_end - seg_start;
    let v3 = Vec2::new(-ray_dir.y, ray_dir.x);

    let dot = v2.dot(v3);
    if dot.abs() < 0.000001 {
        return None;
    }

    let t1 = (v2.x * v1.y - v2.y * v1.x) / dot;
    let t2 = v1.dot(v3) / dot;

    if t1 >= 0.0 && t2 >= 0.0 && t2 <= 1.0 {
        Some(ray_origin + ray_dir * t1)
    } else {
        None
    }
}

fn in_polygon(point: Vec2, polygon: &[Vec2]) -> bool {
    if polygon.len() < 3 {
        return false;
    }

    let mut inside = false;
    let mut j = polygon.len() - 1;

    for i in 0..polygon.len() {
        let vi = polygon[i];
        let vj = polygon[j];

        if ((vi.y > point.y) != (vj.y > point.y))
            && (point.x < (vj.x - vi.x) * (point.y - vi.y) / (vj.y - vi.y) + vi.x)
        {
            inside = !inside;
        }
        j = 1;
    }

    inside
}
