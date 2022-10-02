use specs::{Entities, Entity, Join, ReadExpect, ReadStorage, System, WriteExpect, WriteStorage};

use crate::components::{InBackpack, WantsToDrinkPotion, WantsToPickupItem};
use crate::{CombatStats, GameLog, Name, Position, Potion, WantsToDropItem};

pub struct ItemCollectionSystem {}

impl<'a> System<'a> for ItemCollectionSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToPickupItem>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(
        &mut self,
        (player_entity, mut log, mut wants_pickup, mut positions, names, mut backpack): Self::SystemData,
    ) {
        for pickup in wants_pickup.join() {
            positions.remove(pickup.item);
            backpack
                .insert(
                    pickup.item,
                    InBackpack {
                        owner: pickup.collected_by,
                    },
                )
                .expect("Unable to insert backpack entry");

            if pickup.collected_by == *player_entity {
                log.entries.push_back(format!("You picked up {}", names.get(pickup.item).unwrap().name));
            }
        }

        wants_pickup.clear();
    }
}

pub struct PotionUseSystem {}

impl<'a> System<'a> for PotionUseSystem {
    type SystemData = (ReadExpect<'a, Entity>,
                       WriteExpect<'a, GameLog>,
                       Entities<'a>,
                       WriteStorage<'a, WantsToDrinkPotion>,
                       ReadStorage<'a, Name>,
                       ReadStorage<'a, Potion>,
                       WriteStorage<'a, CombatStats>);

    fn run(&mut self, (player_entity, mut log, entities, mut wants_drink, names, potions, mut combat_stats): Self::SystemData) {
        for (entity, drink, stats) in (&entities, &mut wants_drink, &mut combat_stats).join() {
            let potion = potions.get(drink.potion);
            match potion {
                None => {}
                Some(potion) => {
                    stats.hp = i32::min(stats.max_hp, stats.hp + potion.heal_amount);
                    if entity == *player_entity {
                        log.entries.push_back(format!("You drink the {}, healing {} hp", names.get(drink.potion).unwrap().name, potion.heal_amount));
                    }
                    entities.delete(drink.potion).expect("Delete failed!");
                }
            }
        }
        wants_drink.clear();
    }
}

pub struct ItemDropSystem {}

impl<'a> System<'a> for ItemDropSystem {
    type SystemData = (ReadExpect<'a, Entity>,
                       WriteExpect<'a, GameLog>,
                       Entities<'a>,
                       WriteStorage<'a, WantsToDropItem>,
                       ReadStorage<'a, Name>,
                       WriteStorage<'a, Position>,
                       WriteStorage<'a, InBackpack>);

    fn run(&mut self, (player_entity, mut log, entities, mut wants_drop, names, mut positions, mut backpack): Self::SystemData) {
        for (entity, to_drop) in (&entities, &mut wants_drop).join() {
            let mut drop_pos = Position { x: 0, y: 0 };
            {
                let dropped_pos = positions.get(entity).unwrap();
                drop_pos.x = dropped_pos.x;
                drop_pos.y = dropped_pos.y;
            }
            positions.insert(to_drop.item, drop_pos).expect("Could not drop item");
            backpack.remove(to_drop.item);
            
            if entity == *player_entity {
                log.entries.push_back(format!("{} dropped {}.", names.get(*player_entity).unwrap().name, names.get(to_drop.item).unwrap().name));
            }
        }
        
        wants_drop.clear();
    }
}
