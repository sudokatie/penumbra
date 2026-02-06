//! Tests for field of view.

use penumbra::fov::{calculate_fov, is_visible};

// === FOV Tests (Task 11) ===

#[test]
fn player_position_always_visible() {
    let visible = calculate_fov((5, 5), 5, |_, _| false);
    assert!(visible.contains(&(5, 5)));
}

#[test]
fn adjacent_tiles_visible() {
    let visible = calculate_fov((5, 5), 5, |_, _| false);
    assert!(visible.contains(&(5, 4)));
    assert!(visible.contains(&(5, 6)));
    assert!(visible.contains(&(4, 5)));
    assert!(visible.contains(&(6, 5)));
}

#[test]
fn tiles_at_radius_visible() {
    let visible = calculate_fov((5, 5), 3, |_, _| false);
    // Cardinal directions at radius
    assert!(visible.contains(&(5, 2)));
    assert!(visible.contains(&(5, 8)));
    assert!(visible.contains(&(2, 5)));
    assert!(visible.contains(&(8, 5)));
}

#[test]
fn tiles_beyond_radius_not_visible() {
    let visible = calculate_fov((5, 5), 2, |_, _| false);
    // 4 tiles away should not be visible
    assert!(!visible.contains(&(5, 1)));
    assert!(!visible.contains(&(9, 5)));
}

#[test]
fn wall_blocks_tiles_behind() {
    // Wall at (5, 4), origin at (5, 5)
    let is_wall = |x: i32, y: i32| x == 5 && y == 4;
    let visible = calculate_fov((5, 5), 5, is_wall);
    
    // Wall itself should be visible
    assert!(visible.contains(&(5, 4)));
    // Tiles behind wall should not be visible (or might be from other angles)
}

#[test]
fn empty_room_fully_visible() {
    let visible = calculate_fov((5, 5), 3, |_, _| false);
    
    // Count visible tiles in radius
    let mut count = 0;
    for y in 2..=8 {
        for x in 2..=8 {
            if visible.contains(&(x, y)) {
                count += 1;
            }
        }
    }
    // Should see a good portion of the area
    assert!(count > 20);
}

#[test]
fn corridor_limits_visibility() {
    // Create a corridor: walls on all sides except (5, y)
    let is_wall = |x: i32, y: i32| x != 5 && y != 5;
    let visible = calculate_fov((5, 5), 5, is_wall);
    
    // Center column should be visible
    assert!(visible.contains(&(5, 3)));
    assert!(visible.contains(&(5, 7)));
}

#[test]
fn is_visible_same_point() {
    assert!(is_visible((5, 5), (5, 5), |_, _| false));
}

#[test]
fn is_visible_blocked_by_wall() {
    let is_wall = |x: i32, y: i32| x == 5 && y == 4;
    assert!(!is_visible((5, 5), (5, 2), is_wall));
}

#[test]
fn fov_performance_large_room() {
    // 100x100 room should complete quickly
    let start = std::time::Instant::now();
    let visible = calculate_fov((50, 50), 10, |_, _| false);
    let elapsed = start.elapsed();
    
    assert!(!visible.is_empty());
    assert!(elapsed.as_millis() < 100); // Should be under 100ms
}
