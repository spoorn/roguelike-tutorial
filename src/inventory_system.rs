use specs::{Entities, Entity, Join, ReadExpect, ReadStorage, System, WriteExpect, WriteStorage};

use crate::{CombatStats, GameLog, Name, Position, ProvidesHealing, WantsToDropItem, WantsToUseItem};
use crate::components::{Consumable, InBackpack, WantsToPickupItem};

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

pub struct ItemUseSystem {}

impl<'a> System<'a> for ItemUseSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        WriteStorage<'a, WantsToUseItem>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, ProvidesHealing>,
        ReadStorage<'a, Consumable>,
        WriteStorage<'a, CombatStats>,
    );

    fn run(
        &mut self,
        (player_entity, mut log, entities, mut wants_use, names, provides_healing, consumables, mut combat_stats): Self::SystemData,
    ) {
        for (entity, use_item, stats) in (&entities, &mut wants_use, &mut combat_stats).join() {
            let healing_item = provides_healing.get(use_item.item);
            match healing_item {
                None => {}
                Some(potion) => {
                    stats.hp = i32::min(stats.max_hp, stats.hp + potion.heal_amount);
                    if entity == *player_entity {
                        log.entries.push_back(format!(
                            "You drink the {}, healing {} hp",
                            names.get(use_item.item).unwrap().name,
                            potion.heal_amount
                        ));
                    }
                }
            }
    
            // Delete consumables
            let consumable = consumables.get(use_item.item);
            match consumable {
                None => {}
                Some(_) => {
                    entities.delete(use_item.item).expect("Delete failed");
                }
            }
        }
        wants_use.clear();
    }
}

pub struct ItemDropSystem {}

impl<'a> System<'a> for ItemDropSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        WriteStorage<'a, WantsToDropItem>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(
        &mut self,
        (player_entity, mut log, entities, mut wants_drop, names, mut positions, mut backpack): Self::SystemData,
    ) {
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
                log.entries.push_back(format!(
                    "{} dropped {}.",
                    names.get(*player_entity).unwrap().name,
                    names.get(to_drop.item).unwrap().name
                ));
            }
        }

        wants_drop.clear();
    }
}
