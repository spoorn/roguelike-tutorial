#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashMap;

use bounded_vec_deque::BoundedVecDeque;
use rltk::{BResult, GameState, Point, RandomNumberGenerator, Rltk, RltkBuilder, VirtualKeyCode};
use specs::{Join, RunNow, World, WorldExt};

use crate::components::{BlocksTile, CombatStats, Consumable, InBackpack, InflictsDamage, Item, Monster, MovementSpeed, Name, Player, Position, ProvidesHealing, Ranged, Renderable, SerializationHelper, SerializeMe, SufferDamage, Viewshed, WantsToDropItem, WantsToMelee, WantsToPickupItem, WantsToUseItem};
use crate::damage_system::DamageSystem;
use crate::gamelog::GameLog;
use crate::gui::{MainMenuResult, MainMenuSelection};
use crate::inventory_system::{ItemCollectionSystem, ItemDropSystem, ItemUseSystem};
use crate::keys_util::KeyPress;
use crate::map::{draw_map, Map};
use crate::map_indexing_system::MapIndexingSystem;
use crate::melee_combat_system::MeleeCombatSystem;
use crate::monster_ai_system::MonsterAI;
use crate::player::player_input;
use crate::visibility_system::VisibilitySystem;
use specs::saveload::{SimpleMarker, SimpleMarkerAllocator};

mod components;
mod damage_system;
mod gamelog;
mod gui;
mod inventory_system;
mod keys_util;
mod map;
mod map_indexing_system;
mod melee_combat_system;
mod monster_ai_system;
mod movement_util;
mod player;
mod rect;
mod spawner;
mod visibility_system;
mod save_load_system;

macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = ::std::collections::HashMap::new();
         $( map.insert($key, $val); )*
         map
    }}
}

#[derive(Debug)]
pub struct Client {
    pub show_inventory: bool,
    pub drop_inventory: bool,
    pub keys: HashMap<VirtualKeyCode, KeyPress>,
}

impl Default for Client {
    fn default() -> Self {
        Client {
            show_inventory: false,
            drop_inventory: false,
            keys: hashmap![VirtualKeyCode::E => KeyPress::new(100, 500), VirtualKeyCode::I => KeyPress::new(100, 500), VirtualKeyCode::Escape => KeyPress::new(100, 500), VirtualKeyCode::G => KeyPress::new(100, 500)],
        }
    }
}

pub struct State {
    ecs: World,
    client: Client,
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);
        let mut mob = MonsterAI {};
        mob.run_now(&self.ecs);
        let mut mapindex = MapIndexingSystem {};
        mapindex.run_now(&self.ecs);
        let mut melee_combat = MeleeCombatSystem {};
        melee_combat.run_now(&self.ecs);
        let mut damage = DamageSystem {};
        damage.run_now(&self.ecs);
        let mut inventory = ItemCollectionSystem {};
        inventory.run_now(&self.ecs);
        let mut items = ItemUseSystem {};
        items.run_now(&self.ecs);
        let mut drops = ItemDropSystem {};
        drops.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        let mut newrunstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            newrunstate = *runstate;
        }
        
        ctx.cls();
        
        match newrunstate {
            RunState::MainMenu {..} => {
                let result = gui::main_menu(self, ctx);
                match result {
                    MainMenuResult::NoSelection { selected } => {
                        newrunstate = RunState::MainMenu { menu_selection: selected }
                    }
                    MainMenuResult::Selected { selected } => {
                        match selected {
                            MainMenuSelection::NewGame => {
                                println!("new game");
                            }
                            MainMenuSelection::LoadGame => {
                                println!("load game");
                            }
                            MainMenuSelection::Quit => { std::process::exit(0); }
                        }
                    }
                }
            },
            RunState::Running => {
                newrunstate = player_input(self);
                self.run_systems();

                damage_system::delete_the_dead(&mut self.ecs);

                draw_map(&self.ecs, ctx);

                // Renderables
                {
                    let positions = self.ecs.read_storage::<Position>();
                    let renderables = self.ecs.read_storage::<Renderable>();
                    let map = self.ecs.fetch::<Map>();

                    let mut data = (&positions, &renderables).join().collect::<Vec<_>>();
                    data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order));
                    for (pos, render) in data.iter() {
                        let idx = map.xy_idx(pos.x, pos.y);
                        if map.visible_tiles[idx] {
                            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
                        }
                    }
                }

                gui::draw_ui(self, ctx);
            },
            RunState::SaveGame => {
                println!("Saving game");
                saveload_system::save_game(&mut self.ecs);
                newrunstate = RunState::MainMenu { menu_selection: MainMenuSelection::LoadGame };
            }
            _ => {
                panic!("Invalid run state: {:?}", newrunstate);
            }
        }

        *self.ecs.write_resource::<RunState>() = newrunstate;
    }
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum RunState {
    Paused,
    Running,
    SaveGame,
    MainMenu { menu_selection: gui::MainMenuSelection }
}

fn main() -> BResult<()> {
    let mut context = RltkBuilder::simple80x50().with_title("Roguelike Tutorial").build()?;
    context.with_post_scanlines(true);

    // World
    let mut world = World::new();
    world.register::<Position>();
    world.register::<Renderable>();
    world.register::<Player>();
    world.register::<Viewshed>();
    world.register::<Monster>();
    world.register::<Name>();
    world.register::<MovementSpeed>();
    world.register::<BlocksTile>();
    world.register::<CombatStats>();
    world.register::<WantsToMelee>();
    world.register::<SufferDamage>();
    world.register::<Item>();
    world.register::<ProvidesHealing>();
    world.register::<WantsToPickupItem>();
    world.register::<InBackpack>();
    world.register::<WantsToUseItem>();
    world.register::<Consumable>();
    world.register::<WantsToDropItem>();
    world.register::<Ranged>();
    world.register::<InflictsDamage>();
    world.register::<SerializationHelper>();
    
    // Serializing entities
    world.register::<SimpleMarker<SerializeMe>>();
    world.insert(SimpleMarkerAllocator::<SerializeMe>::new());

    // RNG
    world.insert(RandomNumberGenerator::new());

    let map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();

    // Player
    let player_entity = spawner::player(&mut world, player_x, player_y);
    // Add the player as an Entity resource itself so it can be referenced from everywhere
    world.insert(player_entity);

    // Monsters
    for (_i, room) in map.rooms.iter().skip(1).enumerate() {
        spawner::spawn_room(&mut world, room);
    }

    // Map
    world.insert(map);
    // Player position as a resource since it's used often
    world.insert(Point::new(player_x, player_y));
    // Game logs
    let mut entries = BoundedVecDeque::new(127);
    entries.push_back("Welcome to spoorn's dungeon (:<".to_string());
    world.insert(GameLog { entries });
    
    // RunState
    world.insert(RunState::Running);

    // GameState
    let gs = State {
        ecs: world,
        client: Client::default(),
    };
    rltk::main_loop(context, gs)
}
