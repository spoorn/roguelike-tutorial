use std::cmp::{max, min};
use std::ops::DerefMut;

use rltk::{Point, Rltk, VirtualKeyCode};
use specs::{Entity, Join, World, WorldExt};

use crate::{CombatStats, MovementSpeed, Player, Position, RunState, State, Viewshed, WantsToMelee};
use crate::map::Map;
use crate::movement_util::can_move;

// Below cannot be in a system because they require context outside the ECS, such as Rltk
pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();
    let combat_stats = ecs.read_storage::<CombatStats>();
    let entities = ecs.entities();
    let map = ecs.fetch::<Map>();

    for (entity, _player, pos, viewshed) in (&entities, &mut players, &mut positions, &mut viewsheds).join() {
        let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);
        
        // Targets
        for potential_target in map.tile_content[destination_idx].iter() {
            let target = combat_stats.get(*potential_target);
            if let Some(_target) = target {
                // Player can target multiple different targets, so we dynamically add it here
                wants_to_melee.insert(entity, WantsToMelee { target: *potential_target }).expect("Add target failed");
                return;  // So we don't move after attacking
            }
        }
        
        // Move
        if !map.blocked[destination_idx] {
            pos.x = min(79, max(0, pos.x + delta_x));
            pos.y = min(49, max(0, pos.y + delta_y));

            viewshed.dirty = true;

            // Update player position resource
            let mut ppos = ecs.write_resource::<Point>();
            ppos.x = pos.x;
            ppos.y = pos.y;
        }
    }
}

pub fn player_input_free_movement(gs: &mut State) {
    //let mut key = ctx.key;
    //let mut client = &mut gs.client;

    // For constraining speed of free movement
    //let player = gs.ecs.read_storage::<Player>().fetched_entities().join().next().unwrap();  // Assumes 1 player
    {
        let player = gs.ecs.write_resource::<Entity>();

        // Doing it all in one line beats the borrow checker here
        if !can_move(gs.ecs.write_storage::<MovementSpeed>().get_mut(*player).unwrap().deref_mut()) {
            return ()
        }
    }
    // let current_time = SystemTime::now();
    // if let Some(last_key_time) = client.last_key_time {
    //     let elapsed = current_time.duration_since(last_key_time).unwrap();
    // 
    //     // Constrains speed of movement
    //     if elapsed < Duration::from_millis(60) {
    //         return ()
    //     }
    // 
    //     // if let Some(last_key) = client.last_key_pressed {
    //     //     let input = rltk::INPUT.lock();
    //     //     // Bypass keyboard repeat delay
    //     //     if input.is_key_pressed(last_key) {
    //     //         println!("same");
    //     //         // Simulate keyboard repeat delay, but we can control the time ourselves
    //     //         if !client.checked_first_press && elapsed < Duration::from_millis(10000) {
    //     //             client.checked_first_press = true;
    //     //             println!("### skip");
    //     //             return RunState::Paused
    //     //         }
    //     //         client.last_key_pressed = key
    //     //     }
    //     // }
    // }

    // Update last key time
    //client.last_key_pressed = key;
    //client.last_key_time = Some(current_time);

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
