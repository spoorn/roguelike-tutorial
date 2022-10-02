use rltk::Point;
use specs::prelude::*;

use crate::{Map, Monster, MovementSpeed, Name, Position, Viewshed, WantsToMelee};
use crate::movement_util::can_move;

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    type SystemData = (WriteExpect<'a, Map>, WriteStorage<'a, Viewshed>, WriteStorage<'a, Position>, ReadStorage<'a, Monster>, ReadExpect<'a, Point>, ReadStorage<'a, Name>, WriteStorage<'a, MovementSpeed>, WriteStorage<'a, WantsToMelee>, ReadExpect<'a, Entity>, Entities<'a>);

    fn run(&mut self, (mut map, mut viewshed, mut pos, monster, player_pos, name, mut movement_speed, mut wants_to_melee, player_entity, entities): Self::SystemData) {
        for (viewshed, pos, _monster, _name, movement_speed, entity) in (&mut viewshed, &mut pos, &monster, &name, &mut movement_speed, &entities).join() {
            if viewshed.visible_tiles.contains(&*player_pos) {
                // Monster movement speed
                // Also used to limit attack speed for now
                if !can_move(movement_speed) {
                    continue;
                }
                
                // Stop moving if already next to player, then attack
                let distance = rltk::DistanceAlg::Pythagoras.distance2d(Point::new(pos.x, pos.y), *player_pos);
                if distance < 1.5 {
                    // Attack goes here
                    wants_to_melee.insert(entity, WantsToMelee { target: *player_entity }).expect("Could not add target");
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