use specs::{Entities, Join, ReadStorage, System, WriteExpect, WriteStorage};

use crate::{CombatStats, GameLog, Name, SufferDamage, WantsToMelee};

pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToMelee>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
    );

    fn run(&mut self, (entities, mut log, mut wants_melee, names, combat_stats, mut suffer_damage): Self::SystemData) {
        for (_entity, wants_melee, name, stats) in (&entities, &wants_melee, &names, &combat_stats).join() {
            if stats.hp > 0 {
                let target_stats = combat_stats.get(wants_melee.target).unwrap();
                if target_stats.hp > 0 {
                    let target_name = names.get(wants_melee.target).unwrap();
                    let damage = i32::max(0, stats.power - target_stats.defense);

                    if damage == 0 {
                        log.entries.push_back(format!("{} is unable to hurt {}", &name.name, &target_name.name));
                    } else {
                        log.entries.push_back(format!("{} hits {}, for {} hp.", &name.name, &target_name.name, damage));
                        SufferDamage::new_damage(&mut suffer_damage, wants_melee.target, damage);
                    }
                }
            }
        }

        wants_melee.clear();
    }
}
