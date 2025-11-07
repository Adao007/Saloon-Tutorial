# aim.rs — Developer guide

This document explains the purpose, key data types, algorithms, and interaction points inside `src/gameplay/aim.rs`.

It is written for contributors who want to understand how player aiming, cone-of-vision, visibility testing, and fog discovery work in the Saloon tutorial project.

## High-level overview

`aim.rs` implements:
- mouse-to-world tracking (`MousePos`).
- player aiming/rotation logic (`rotate_aim`).
- visibility cone definition (`VisibilityCone` component).
- a raycast-based visibility polygon generator (`calculate_visibility_polygon`).
- a point-in-polygon visibility test (`in_polygon`) used by the fog/discovery system.
- simple fog state updates (`update_fog`) and sprite visual changes (`apply_fog_visuals`).
- debug drawing of visibility rays and polygon (`draw_visibility`).

The visibility algorithm casts rays from the player's origin toward obstacle vertices and across the cone arc, finds nearest intersections, and builds a boundary of points representing the visible region.

## Key types and resources

- `MousePos` (Resource)
  - position: `Vec2` — last known mouse position in world coordinates.
  - Updated by `get_mouse_position` which translates window cursor positions into world-space via `camera.viewport_to_world`.

- `VisibilityCone` (Component)
  - `range: f32` — how far the cone reaches.
  - `angle: f32` — cone angular width, in radians.
  - `direction: Vec2` — normalized forward vector of the cone (unit vector pointing where player aims).

- `Fog` and `Obstacle`
  - `Fog` stores `visible` and `discovered` flags for each entity with fog behavior (see `update_fog`).
  - `Obstacle` is defined in `world.rs` and provides a polygon (list of local-space vertices) used for occlusion.

## Core functions and what they do

### get_mouse_position
- Reads the window cursor position and converts it to a world-space point using the camera and its transform.
- Stores the result in `MousePos`.
- Edge cases: if the cursor is outside the window, no update occurs.

### rotate_aim
- Finds the intended aim direction toward the cursor and rotates the player transform smoothly toward it.
- Uses dot products to compute the rotation angle and clamps by `rotation_speed * delta_time` to limit instantaneous rotation.
- Updates `VisibilityCone.direction` to match the player's forward vector.

### ray_segment_interaction
- Low-level geometry helper: computes the intersection point between a ray and a line segment.
- Inputs: `ray_origin`, `ray_dir` (direction vector), `seg_start`, `seg_end`.
- Returns `Some(Vec2)` intersection point if ray hits the segment in front of the origin and within the segment bounds; otherwise `None`.
- Notes: numerical threshold used to detect parallel lines.

### in_polygon
- Standard ray-crossing parity test (even/odd rule) for point-in-polygon.
- Returns `true` when the point is inside the polygon.
- Requires polygon to be a closed loop of boundary points (vertex order matters).
- Edge cases: points exactly on an edge or vertex depend on inequality checks and floating-point precision.

### calculate_visibility_polygon
- Core algorithm to compute the set of world-space boundary points describing what the player can see.
- Steps:
  1. Compute the cone center angle from `cone.direction` using `atan2`.
  2. Collect angles to every obstacle vertex (world-space), convert them to deltas relative to the cone center, and keep only those inside the cone's angular span.
  3. For each kept vertex angle, push a tiny neighborhood (delta - eps, delta, delta + eps) to ensure rays just on the edges are correctly sampled.
  4. Add the cone boundary deltas (`-half_cone` and `+half_cone`) and additional uniform samples along the arc (`num_samples`) to cover open regions.
  5. Sort deltas and deduplicate them.
  6. For each delta, compute the absolute angle, cast a ray of length `cone.range`, intersect it against all obstacle edges via `ray_segment_interaction` and keep the nearest intersection point (or the endpoint at `cone.range` if no hit).
  7. Return the list of boundary points in order (the visibility boundary).

- Implementation notes:
  - The function uses an angle-normalization helper to keep delta values in [-π, π]. That avoids sort-order flips when angles straddle the ±π boundary.
  - `num_samples` controls the angular resolution; increasing it improves accuracy at the cost of more raycasts.
  - The polygon returned is the list of sample hits in angular order (useful for rendering). `update_fog` constructs an origin-centered fan for the point-in-polygon test to avoid closing chords.

### is_angle_in_cone (unused)
- Helper that returns whether a given angle lies inside the cone; it is equivalent to checking the absolute normalized delta against `half_cone`.
- Currently flagged as dead code (left in place for reference).

### normalize_angle
- Wraps angles into [-π, π] by adding or subtracting 2π as needed.
- This helps compare angles robustly.

### update_fog
- Top-level system that updates `Fog` components based on the latest visibility polygon.
- For each player it:
  - Calculates the visibility polygon.
  - Builds an origin-centered fan (player position + polygon points) and uses `in_polygon` to test if each entity's position is inside the fan.
  - Sets `fog.visible` and `fog.discovered` accordingly.
- Note: Currently it tests the single reference point of the entity (transform translation). If entities are larger than a point, their center could be occluded while part of the entity is visible.

### apply_fog_visuals
- Reacts to `Fog` changes and modifies the `Sprite` color to one of:
  - original color when visible,
  - desaturated color when discovered but not currently visible,
  - fully transparent black for never-discovered.

- Color conversions use `to_srgba()` and a simple desaturation blend.

### draw_visibility
- Debugging helper that draws:
  - The visibility polygon edges (note: the implementation was updated to avoid drawing the closing chord between the first and last samples because that chord can cross occluders and misleadingly show occluded space),
  - Lines (rays) from the player to each sample point,
  - The cone direction arrow,
  - Obstacle vertices.
- Useful to visualize exactly what rays are being cast and where the polygon boundary lies.

## How to interact with the functionality

1. Run the game

   ```bash
   cargo run
   ```

   The app will create the Bevy window. Use the mouse to aim; the player rotation and `VisibilityCone.direction` will follow the mouse.

2. Visual debugging

- Enable `draw_visibility` system in the schedule (it is already implemented as a system — check `gameplay.rs` to see whether it's registered). When active, it draws the polygon and rays so you can visually inspect occlusion.
- Watch the console for warnings (unused helpers) but there are no runtime panics related to the visibility code.

3. Tune the visibility

- Increase `num_samples` in `calculate_visibility_polygon` to reduce false-positive visibility at cone edges (e.g., try 32 or 64). This increases raycasts per frame.
- Increase EPS offsets when adding delta ±0.00001 to better handle edge cases when a vertex lies exactly on the ray.
- To test occlusion more strictly per object, replace the `in_polygon` center test with a per-entity raycast to a few sample points around the object's bounds.

4. Fog behavior

- `update_fog` sets `Fog.visible` and `Fog.discovered`. `apply_fog_visuals` applies colors.
- To change fog appearance, adjust `desaturate_color` or the alpha values in `apply_fog_visuals`.

## Behavioural edge cases and gotchas

- Closing chord artifact: when using a polygon boundary without the origin, the straight line between the last and first sample may pass through occluders and incorrectly mark things visible. To avoid that, a fan polygon (origin + points) is used for the parity test, while rendering only consecutive boundary edges.

- Sparse sampling: if `num_samples` is too small the polygon will skip narrow gaps around occluders, producing a "fill" effect that marks occluded objects visible. Increase samples or add per-vertex offsets to improve fidelity.

- Precision: floats and comparisons in `in_polygon` mean object positions exactly on edges behave unpredictably. If you need deterministic behavior for on-edge cases, explicitly handle `on_edge` cases in the parity test or use a small epsilon.

- Object size: testing only an entity's translation means entities with extents might be partially visible even if the center is occluded. For more accurate detection, test several sample points around the object's bounds or cast a cone of rays toward the object's AABB.

- Performance: the algorithm runs O(samples * edges) ray-segment intersection tests per player per frame. For many obstacles or high sample counts, this can become expensive. Consider:
  - spatial partitioning (quad-tree, grid) to only test nearby obstacles,
  - lowering sample counts at distance,
  - switching to a GPU-based mask/shader approach,
  - batching raycasts or caching results for static geometry.

## Suggested improvements and alternatives

1. Per-entity raycasts for accuracy
   - Instead of a global polygon + parity test, raycast directly to each entity center or to multiple points on its bounds to determine occlusion. This is usually cheaper when the number of entities to check is small compared to the number of polygon samples.

2. Spatial acceleration
   - Use a simple grid or BVH to filter obstacle edges before performing expensive ray-segment intersections.

3. Shader-based mask (GPU)
   - Render the visibility polygon into a mask (texture or stencil) on the GPU and apply it as a fullscreen quad material. This offloads work to the GPU and simplifies per-pixel visibility but is more complex to implement and requires careful handling of anti-aliasing and edge cases.

4. Triangulation approach for fog
   - Triangulate the viewport minus the visibility polygon and render triangles as a single mesh. This reduces per-frame entity churn and works well when the polygon is well-formed.

5. Winding number test
   - For more robust inside/outside tests (e.g., with polygons that might be self-intersecting), consider a winding-number test instead of parity. It can be more tolerant to malformed polygons.

## Quick debugging checklist

- Turn on `draw_visibility` to see rays and samples.
- If objects behind walls are visible:
  1. Increase `num_samples` in `calculate_visibility_polygon`.
  2. Verify obstacle vertices are in the expected world positions (inspect transforms).
  3. Temporarily per-entity raycast from player to entity center — if that hides the object, your polygon sampling is too sparse.
  4. If you see a straight line connecting distant polygon points crossing a wall, ensure `update_fog` uses an origin-centered fan when testing `in_polygon` (it currently does).

## Contract (inputs, outputs, error modes)

- Inputs:
  - Player transform and `VisibilityCone` component.
  - `Obstacle` entities with `Transform` and polygon vertices.
  - `Fog` components on entities to update discovery/visibility state.

- Outputs:
  - `Fog.visible` and `Fog.discovered` flags per entity.
  - Visual debug geometry via `draw_visibility`.
  - Sprite color changes via `apply_fog_visuals`.

- Error modes:
  - False-positives near sparse-sampled cone edges.
  - Points on polygon edges ambiguous due to FP precision.
  - Performance degradation with many obstacles and high sample counts.

## Small, safe changes you can try now

- Increase `num_samples` in `calculate_visibility_polygon` (e.g., `let num_samples = 32;`).
- For immediate per-entity accuracy, modify `update_fog` to perform a single `ray_segment_interaction` test along the vector from player to `obj_pos`, checking if the ray hits anything closer than `obj_pos`.
- Log debug info for a particular entity to confirm whether it is actually inside the polygon or being misclassified.

## Where to look in code

- `get_mouse_position` — mouse to world conversion.
- `rotate_aim` — player rotation and cone direction update.
- `calculate_visibility_polygon` — algorithm to produce boundary points for visible area.
- `update_fog` — maps polygon → per-entity fog flags; constructs the origin-centered fan for the parity test.
- `apply_fog_visuals` — sprite color updates when fog state changes.
- `draw_visibility` — debug visualization.

---

If you'd like, I can also:
- add this file to the repository (done) and open a follow-up PR that increases default `num_samples` or adds a debug flag to toggle `draw_visibility` at runtime;
- implement a per-entity raycast fallback for entities near the edge of the polygon;
- or write a small unit test for `ray_segment_interaction` and `in_polygon` to lock behavior.

Tell me which next step you prefer.