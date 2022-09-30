use std::time::{Duration, SystemTime};
use rltk::{console, Point};
use specs::prelude::*;
use crate::{Map, Monster, MovementSpeed, Name, Position, Viewshed};
use crate::movement_util::can_move;

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    type SystemData = (WriteExpect<'a, Map>, WriteStorage<'a, Viewshed>, WriteStorage<'a, Position>, ReadStorage<'a, Monster>, ReadExpect<'a, Point>, ReadStorage<'a, Name>, WriteStorage<'a, MovementSpeed>);

    fn run(&mut self, (mut map, mut viewshed, mut pos, monster, player_pos, name, mut movement_speed): Self::SystemData) {
        for (viewshed, pos, monster, name, movement_speed) in (&mut viewshed, &mut pos, &monster, &name, &mut movement_speed).join() {
            if viewshed.visible_tiles.contains(&*player_pos) {
                console::log(&format!("{} shouts insults", name.name));
                
                // Monster movement speed
                if !can_move(movement_speed) {
                    continue;
                }
                
                let path = rltk::a_star_search(
                    map.xy_idx(pos.x, pos.y) as i32,
                    map.xy_idx(player_pos.x, player_pos.y) as i32,
                    &mut *map
                );
                if path.success && path.steps.len() > 1 {
                    pos.x = path.steps[1] as i32 % map.width;
                    pos.y = path.steps[1] as i32 / map.width;
                    viewshed.dirty = true;
                }
            }
        }
    }
}