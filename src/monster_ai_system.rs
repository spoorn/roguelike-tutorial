use rltk::console;
use specs::prelude::*;
use crate::{Monster, Position, Viewshed};

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    type SystemData = (ReadStorage<'a, Viewshed>, WriteStorage<'a, Position>, ReadStorage<'a, Monster>);

    fn run(&mut self, (viewshed, mut pos, monster): Self::SystemData) {
        for (viewshed, pos, monster) in (&viewshed, &mut pos, &monster).join() {
            //console::log("Monster considers their own existence");
        }
    }
}