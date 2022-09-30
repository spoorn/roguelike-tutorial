use rltk::{console, Point};
use specs::prelude::*;
use crate::{Monster, Name, Position, Viewshed};

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    type SystemData = (ReadStorage<'a, Viewshed>, WriteStorage<'a, Position>, ReadStorage<'a, Monster>, ReadExpect<'a, Point>, ReadStorage<'a, Name>);

    fn run(&mut self, (viewshed, mut pos, monster, player_pos, name): Self::SystemData) {
        for (viewshed, pos, monster, name) in (&viewshed, &mut pos, &monster, &name).join() {
            if viewshed.visible_tiles.contains(&*player_pos) {
                console::log(&format!("{} shouts insults", name.name));
            }
        }
    }
}