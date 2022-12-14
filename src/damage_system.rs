use specs::{Entity, Join, System, World, WorldExt, WriteStorage};

use crate::{CombatStats, GameLog, Player, SufferDamage};

pub struct DamageSystem {}

impl<'a> System<'a> for DamageSystem {
    type SystemData = (WriteStorage<'a, CombatStats>, WriteStorage<'a, SufferDamage>);

    fn run(&mut self, (mut combat_stats, mut suffer_damage): Self::SystemData) {
        for (mut stats, damage) in (&mut combat_stats, &suffer_damage).join() {
            stats.hp -= damage.amount.iter().sum::<i32>();
        }
        suffer_damage.clear();
    }
}

pub fn delete_the_dead(ecs: &mut World) {
    let mut dead: Vec<Entity> = Vec::new();
    // using a scope to make the borrow checker happy
    {
        let combat_stats = ecs.read_storage::<CombatStats>();
        let players = ecs.read_storage::<Player>();
        let entities = ecs.entities();
        let mut log = ecs.fetch_mut::<GameLog>();
        for (entity, stats) in (&entities, &combat_stats).join() {
            if stats.hp < 1 {
                let player = players.get(entity);
                match player {
                    None => {
                        dead.push(entity);
                    }
                    Some(_) => {
                        log.entries.push_back("You are dead!".to_string());
                    }
                }
            }
        }
    }

    for victim in dead {
        ecs.delete_entity(victim).expect("Unable to delete");
    }
}
