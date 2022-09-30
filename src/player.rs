use std::cmp::{max, min};
use std::time::{Duration, SystemTime};

use rltk::{Rltk, VirtualKeyCode};
use specs::{Join, World, WorldExt};

use crate::map::Map;
use crate::{Player, Position, RunState, State, TileType, Viewshed};

// Below cannot be in a system because they require context outside the ECS, such as Rltk
pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let map = ecs.fetch::<Map>();

    for (_player, pos, viewshed) in (&mut players, &mut positions, &mut viewsheds).join() {
        let destination_edx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);
        if map.tiles[destination_edx] != TileType::Wall {
            pos.x = min(79, max(0, pos.x + delta_x));
            pos.y = min(49, max(0, pos.y + delta_y));

            viewshed.dirty = true;
        }
    }
}

pub fn player_input_free_movement(gs: &mut State) {
    //let mut key = ctx.key;
    let mut client = &mut gs.client;

    // For constraining speed of free movement
    let current_time = SystemTime::now();
    if let Some(last_key_time) = client.last_key_time {
        let elapsed = current_time.duration_since(last_key_time).unwrap();

        // Constrains speed of movement
        if elapsed < Duration::from_millis(60) {
            return ()
        }

        // if let Some(last_key) = client.last_key_pressed {
        //     let input = rltk::INPUT.lock();
        //     // Bypass keyboard repeat delay
        //     if input.is_key_pressed(last_key) {
        //         println!("same");
        //         // Simulate keyboard repeat delay, but we can control the time ourselves
        //         if !client.checked_first_press && elapsed < Duration::from_millis(10000) {
        //             client.checked_first_press = true;
        //             println!("### skip");
        //             return RunState::Paused
        //         }
        //         client.last_key_pressed = key
        //     }
        // }
    }

    // Update last key time
    //client.last_key_pressed = key;
    client.last_key_time = Some(current_time);

    // Free movement
    let input = rltk::INPUT.lock();
    if input.is_key_pressed(VirtualKeyCode::Left) {
        try_move_player(-1, 0, &mut gs.ecs)
    }
    if input.is_key_pressed(VirtualKeyCode::Right) {
        try_move_player(1, 0, &mut gs.ecs)
    }
    if input.is_key_pressed(VirtualKeyCode::Up) {
        try_move_player(0, -1, &mut gs.ecs)
    }
    if input.is_key_pressed(VirtualKeyCode::Down) {
        try_move_player(0, 1, &mut gs.ecs)
    }
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    // Player movement
    match ctx.key {
        None => { return RunState::Paused }
        Some(key) => match key {
            VirtualKeyCode::Left | VirtualKeyCode::Numpad4 | VirtualKeyCode::A => {
                try_move_player(-1, 0, &mut gs.ecs)
            }
    
            VirtualKeyCode::Right | VirtualKeyCode::Numpad6 | VirtualKeyCode::D => {
                try_move_player(1, 0, &mut gs.ecs)
            }
    
            VirtualKeyCode::Up | VirtualKeyCode::Numpad8 | VirtualKeyCode::W => {
                try_move_player(0, -1, &mut gs.ecs)
            }
    
            VirtualKeyCode::Down | VirtualKeyCode::Numpad2 | VirtualKeyCode::S => {
                try_move_player(0, 1, &mut gs.ecs)
            }
            _ => { return RunState::Paused }
        },
    }
    
    RunState::Running
}
