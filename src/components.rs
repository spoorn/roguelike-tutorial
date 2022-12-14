use std::time::SystemTime;

use rltk::RGB;
use serde::{Deserialize, Serialize};
use specs::prelude::*;
use specs::saveload::ConvertSaveload;
use specs_derive::Component;

#[derive(Component, ConvertSaveload)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, ConvertSaveload)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub fg: RGB,
    pub bg: RGB,
    pub render_order: i32,
}

#[derive(Component, Debug, ConvertSaveload)]
pub struct Name {
    pub name: String,
}

#[derive(Component, Debug, Serialize, Deserialize)]
pub struct BlocksTile {}

#[derive(Component, Debug, Serialize, Deserialize)]
pub struct Item {}

#[derive(Component, Debug, Clone, ConvertSaveload)]
pub struct InBackpack {
    pub owner: Entity,
}

#[derive(Component, Debug, Clone, ConvertSaveload)]
pub struct WantsToPickupItem {
    pub collected_by: Entity,
    pub item: Entity,
}

#[derive(Component, Debug, ConvertSaveload)]
pub struct ProvidesHealing {
    pub heal_amount : i32
}

#[derive(Component, Debug, Serialize, Deserialize)]
pub struct Consumable {}

#[derive(Component, Debug, ConvertSaveload)]
pub struct WantsToUseItem {
    pub item: Entity,
}

#[derive(Component, Debug, Clone, ConvertSaveload)]
pub struct WantsToDropItem {
    pub item: Entity,
}

#[derive(Component, Debug, ConvertSaveload)]
pub struct MovementSpeed {
    pub min_delay_ms: u64,
    pub last_move_time: Option<SystemTime>,
}

#[derive(Component, Debug, ConvertSaveload)]
pub struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub power: i32,
}

#[derive(Component, Debug, Clone, ConvertSaveload)]
pub struct WantsToMelee {
    pub target: Entity,
}

#[derive(Component, Debug, ConvertSaveload)]
pub struct Ranged {
    pub range : i32
}

#[derive(Component, Debug, ConvertSaveload)]
pub struct InflictsDamage {
    pub damage : i32
}

#[derive(Component, Debug, ConvertSaveload)]
pub struct SufferDamage {
    pub amount: Vec<i32>,
}

impl SufferDamage {
    pub fn new_damage(store: &mut WriteStorage<SufferDamage>, victim: Entity, amount: i32) {
        if let Some(suffering) = store.get_mut(victim) {
            suffering.amount.push(amount);
        } else {
            let dmg = SufferDamage { amount: vec![amount] };
            store.insert(victim, dmg).expect("Unable to insert damage");
        }
    }
}

#[derive(Component, Debug, Serialize, Deserialize)]
pub struct Player {}

#[derive(Component, ConvertSaveload)]
pub struct Viewshed {
    pub visible_tiles: Vec<rltk::Point>,
    pub range: i32,
    pub dirty: bool,
}

#[derive(Component, Debug, Serialize, Deserialize)]
pub struct Monster {}

// Serialization helper code. We need to implement ConvertSaveload for each type that contains an
// Entity.

pub struct SerializeMe;

// Special component that exists to help serialize the game data
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SerializationHelper {
    pub map : super::map::Map
}
