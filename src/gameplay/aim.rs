use super::{
    player::Player,
    world::{Fog, Obstacle},
};
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

pub fn rotate_aim(
    mut player_query: Query<(&mut Transform, &mut VisibilityCone), With<Player>>,
    mouse_pos: Res<MousePos>,
    time: Res<Time>,
) {
    let rotation_speed = f32::to_radians(360.0);
    let aim_translation = mouse_pos.position.xy();
    for (mut player_transform, mut cone) in &mut player_query {
        let player_forward = (player_transform.rotation * Vec3::Y).xy();
        let to_aim = (aim_translation - player_transform.translation.xy()).normalize();
        let forward_dot = player_forward.dot(to_aim);
        if (forward_dot - 1.0).abs() < f32::EPSILON {
            // Update cone anyway
            cone.direction = player_forward;
            continue;
        }

        let player_right = (player_transform.rotation * Vec3::X).xy();
        let right_dot = player_right.dot(to_aim);
        let rotation_sign = -f32::copysign(1.0, right_dot);
        let max_angle = ops::acos(forward_dot.clamp(-1.0, 1.0));

        let rotation_angle = rotation_sign * (rotation_speed * time.delta_secs()).min(max_angle);
        player_transform.rotate_z(rotation_angle);

        // Update cone direction to match aim
        cone.direction = (player_transform.rotation * Vec3::Y).xy();
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
        j = i;
    }

    inside
}

fn calculate_visibility_polygon(
    origin: Vec2,
    obstacles: &Query<(&Transform, &Obstacle)>,
    cone: &VisibilityCone,
) -> Vec<Vec2> {
    // We'll normalize angles relative to the cone center to avoid wrap issues
    let mut deltas: Vec<f32> = Vec::new();

    // Calculate cone angle range
    let cone_center_angle = cone.direction.y.atan2(cone.direction.x);
    let half_cone = cone.angle / 2.0;

    // Collect obstacle corner angles (only within cone), as deltas around center
    for (transform, obstacle) in obstacles.iter() {
        let obs_pos = transform.translation.truncate();

        for vertex in &obstacle.vertices {
            let world_vertex = obs_pos + *vertex;
            let to_vertex = world_vertex - origin;

            if to_vertex.length() <= cone.range {
                let angle = to_vertex.y.atan2(to_vertex.x);
                let delta = normalize_angle(angle - cone_center_angle);

                // Only add if within cone angle
                if delta.abs() <= half_cone {
                    deltas.push(delta - 0.00001);
                    deltas.push(delta);
                    deltas.push(delta + 0.00001);
                }
            }
        }
    }

    // Add cone boundary deltas
    deltas.push(-half_cone);
    deltas.push(half_cone);

    // Add additional samples along the cone arc (in delta space)
    let num_samples = 16;
    for i in 0..num_samples {
        let t = i as f32 / (num_samples - 1) as f32;
        let delta = -half_cone + t * (half_cone - (-half_cone));
        deltas.push(delta);
    }

    // Sort and deduplicate deltas
    deltas.sort_by(|a, b| a.partial_cmp(b).unwrap());
    deltas.dedup_by(|a, b| (*a - *b).abs() < 0.00001);

    let mut polygon = Vec::new();

    // Cast rays for each delta (angle = center + delta)
    for &delta in &deltas {
        let angle = cone_center_angle + delta;
        let ray_dir = Vec2::new(angle.cos(), angle.sin());
        let mut closest_dist = cone.range;
        let mut closest_point = origin + ray_dir * cone.range;
        
        // Check all obstacle edges
        for (transform, obstacle) in obstacles.iter() {
            let obs_pos = transform.translation.truncate();
            
            for i in 0..obstacle.vertices.len() {
                let v1 = obs_pos + obstacle.vertices[i];
                let v2 = obs_pos + obstacle.vertices[(i + 1) % obstacle.vertices.len()];
                
                if let Some(intersection) = ray_segment_interaction(origin, ray_dir, v1, v2) {
                    let dist = (intersection - origin).length();
                    if dist < closest_dist {
                        closest_dist = dist;
                        closest_point = intersection;
                    }
                }
            }
        }
        
        polygon.push(closest_point);
    }
    
    polygon
}

fn is_angle_in_cone(angle: f32, center_angle: f32, half_cone: f32) -> bool {
    let diff = normalize_angle(angle - center_angle);
    diff.abs() <= half_cone
}

fn normalize_angle(angle: f32) -> f32 {
    let mut a = angle;
    while a > std::f32::consts::PI {
        a -= std::f32::consts::TAU;
    }
    while a < -std::f32::consts::PI {
        a += std::f32::consts::TAU;
    }

    a
}

pub fn update_fog(
    player_query: Query<(&Transform, &VisibilityCone), With<Player>>,
    obstacle_query: Query<(&Transform, &Obstacle)>,
    mut fog_query: Query<(&Transform, &mut Fog)>,
) {
    for (player_transform, cone) in &player_query {
        let player_pos = player_transform.translation.truncate();

        // Calculate the visibility polygon
        let visibility_polygon = calculate_visibility_polygon(player_pos, &obstacle_query, cone);

        // Check each entity with fog
        for (obj_transform, mut fog) in &mut fog_query {
            let obj_pos = obj_transform.translation.truncate();
            // Build a fan-shaped polygon (origin + boundary) so the closing edge
            // doesn't create a long chord between the first and last ray sample
            // that can pass through occluders. Using a fan around the origin
            // yields a proper visibility region for the parity test.
            let mut fan_polygon: Vec<Vec2> = Vec::with_capacity(visibility_polygon.len() + 1);
            fan_polygon.push(player_pos);
            fan_polygon.extend(visibility_polygon.iter().cloned());

            let is_visible = in_polygon(obj_pos, &fan_polygon);
            fog.visible = is_visible;

            // If discovered, mark
            if is_visible {
                fog.discovered = true;
            }
        }
    }
}

pub fn apply_fog_visuals(mut query: Query<(&Fog, &mut Sprite), Changed<Fog>>) {
    for (fog, mut sprite) in &mut query {
        if fog.visible {
            sprite.color = fog.original;
        } else if fog.discovered {
            sprite.color = desaturate_color(fog.original, 0.8);
        } else {
            sprite.color = Color::srgba(0.0, 0.0, 0.0, 0.0);
        }
    }
}

fn desaturate_color(color: Color, amount: f32) -> Color {
    let rgba = color.to_srgba();
    let r = rgba.red;
    let g = rgba.green;
    let b = rgba.blue;

    let luminance = 0.299 * r + 0.587 * g + 0.114 * b;

    Color::srgba(
        r + (luminance - r) * amount,
        g + (luminance - g) * amount,
        b + (luminance - b) * amount,
        0.6,
    )
}

pub fn draw_visibility(
    mut gizmos: Gizmos,
    player_query: Query<(&Transform, &VisibilityCone), With<Player>>,
    obstacle_query: Query<(&Transform, &Obstacle)>,
) {
    for (player_transform, cone) in &player_query {
        let player_pos = player_transform.translation.truncate();
        
        let polygon = calculate_visibility_polygon(
            player_pos,
            &obstacle_query,
            cone,
        );
        
        // Draw visibility polygon edges (just the outer boundary). Don't draw
        // the closing chord between the last and first sample — that chord can
        // pass through occluders and incorrectly reveal objects. Instead draw
        // only consecutive sample edges and the fan lines to the origin.
        if polygon.len() >= 2 {
            for i in 0..(polygon.len() - 1) {
                let p1 = polygon[i];
                let p2 = polygon[i + 1];
                gizmos.line_2d(p1, p2, Color::srgba(1.0, 1.0, 0.0, 0.3));
            }
        }
        
        // Optional: Draw lines from player to polygon points (helps visualize rays)
        // Uncomment only if you want to see the rays

        for point in &polygon {
            // Draw rays from player to polygon samples (visual aid)
            gizmos.line_2d(player_pos, *point, Color::srgba(0.0, 1.0, 1.0, 0.1));
        }
        
        // Draw cone direction arrow (green)
        let arrow_end = player_pos + cone.direction * 2.0;
        gizmos.line_2d(player_pos, arrow_end, Color::srgb(0.0, 1.0, 0.0));
        
        // Draw obstacle vertices
        for (transform, obstacle) in &obstacle_query {
            let obs_pos = transform.translation.truncate();
            for vertex in &obstacle.vertices {
                let world_pos = obs_pos + *vertex;
                gizmos.circle_2d(world_pos, 0.1, Color::srgb(1.0, 0.0, 1.0));
            }
        }
    }
}

