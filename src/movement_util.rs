use std::time::{Duration, SystemTime};
use crate::MovementSpeed;

// Ideally this should be based on tick time, not system time!
pub fn can_move(movement_speed: &mut MovementSpeed) -> bool {
    let current_time = SystemTime::now();
    if let Some(last_move_time) = movement_speed.last_move_time {
        let elapsed = current_time.duration_since(last_move_time).unwrap();

        // Constrains speed of movement
        if elapsed < Duration::from_millis(movement_speed.min_delay_ms) {
            return false
        }
    }

    movement_speed.last_move_time = Some(current_time);
    true
}