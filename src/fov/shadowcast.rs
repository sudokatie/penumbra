//! Shadowcasting FOV algorithm.

use std::collections::HashSet;

/// Calculate visible tiles from origin using shadowcasting.
/// 
/// Uses recursive shadowcasting with 8 octants for symmetric FOV.
pub fn calculate_fov<F>(origin: (i32, i32), radius: u32, is_blocking: F) -> HashSet<(i32, i32)>
where
    F: Fn(i32, i32) -> bool,
{
    let mut visible = HashSet::new();
    visible.insert(origin);

    // Cast shadows in all 8 octants
    for octant in 0..8 {
        cast_light(
            &mut visible,
            origin,
            radius as i32,
            1,
            1.0,
            0.0,
            octant,
            &is_blocking,
        );
    }

    visible
}

/// Check if a single point is visible from origin.
pub fn is_visible<F>(origin: (i32, i32), target: (i32, i32), is_blocking: F) -> bool
where
    F: Fn(i32, i32) -> bool,
{
    let dx = (target.0 - origin.0).abs();
    let dy = (target.1 - origin.1).abs();
    let distance = ((dx * dx + dy * dy) as f32).sqrt();
    
    // Simple raycast
    let steps = distance.ceil() as i32;
    if steps == 0 {
        return true;
    }

    for step in 1..steps {
        let t = step as f32 / steps as f32;
        let x = origin.0 as f32 + (target.0 - origin.0) as f32 * t;
        let y = origin.1 as f32 + (target.1 - origin.1) as f32 * t;
        
        if is_blocking(x.round() as i32, y.round() as i32) {
            return false;
        }
    }

    true
}

/// Transform coordinates based on octant.
fn transform_octant(octant: u8, x: i32, y: i32) -> (i32, i32) {
    match octant {
        0 => (x, y),
        1 => (y, x),
        2 => (y, -x),
        3 => (-x, y),
        4 => (-x, -y),
        5 => (-y, -x),
        6 => (-y, x),
        7 => (x, -y),
        _ => (x, y),
    }
}

/// Recursive shadowcasting for one octant.
#[allow(clippy::too_many_arguments)]
fn cast_light<F>(
    visible: &mut HashSet<(i32, i32)>,
    origin: (i32, i32),
    radius: i32,
    row: i32,
    mut start_slope: f32,
    end_slope: f32,
    octant: u8,
    is_blocking: &F,
) where
    F: Fn(i32, i32) -> bool,
{
    if start_slope < end_slope {
        return;
    }

    let mut next_start_slope = start_slope;

    for current_row in row..=radius {
        let mut blocked = false;
        let dy = -current_row;

        for dx in -current_row..=0 {
            let left_slope = (dx as f32 - 0.5) / (dy as f32 + 0.5);
            let right_slope = (dx as f32 + 0.5) / (dy as f32 - 0.5);

            if start_slope < right_slope {
                continue;
            }
            if end_slope > left_slope {
                break;
            }

            // Transform to actual coordinates
            let (tx, ty) = transform_octant(octant, dx, dy);
            let abs_x = origin.0 + tx;
            let abs_y = origin.1 + ty;

            // Check if within radius
            let distance_sq = dx * dx + dy * dy;
            if distance_sq <= radius * radius {
                visible.insert((abs_x, abs_y));
            }

            // Handle blocking
            if blocked {
                if is_blocking(abs_x, abs_y) {
                    next_start_slope = right_slope;
                    continue;
                } else {
                    blocked = false;
                    start_slope = next_start_slope;
                }
            } else if is_blocking(abs_x, abs_y) && current_row < radius {
                blocked = true;
                cast_light(
                    visible,
                    origin,
                    radius,
                    current_row + 1,
                    start_slope,
                    left_slope,
                    octant,
                    is_blocking,
                );
                next_start_slope = right_slope;
            }
        }

        if blocked {
            break;
        }
    }
}
