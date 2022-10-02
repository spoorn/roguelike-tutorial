#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashMap;

use bounded_vec_deque::BoundedVecDeque;
use rltk::{BResult, GameState, Point, RandomNumberGenerator, Rltk, RltkBuilder, VirtualKeyCode};
use specs::{Join, RunNow, World, WorldExt};

use crate::components::{
    BlocksTile, CombatStats, InBackpack, Item, Monster, MovementSpeed, Name, Player, Position, Potion, Renderable,
    SufferDamage, Viewshed, WantsToMelee, WantsToPickupItem,
};
use crate::damage_system::DamageSystem;
use crate::gamelog::GameLog;
use crate::inventory_system::ItemCollectionSystem;
use crate::keys_util::KeyPress;
use crate::map::{draw_map, Map};
use crate::map_indexing_system::MapIndexingSystem;
use crate::melee_combat_system::MeleeCombatSystem;
use crate::monster_ai_system::MonsterAI;
use crate::player::player_input;
use crate::visibility_system::VisibilitySystem;

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
    pub keys: HashMap<VirtualKeyCode, KeyPress>,
}

impl Default for Client {
    fn default() -> Self {
        Client {
            show_inventory: false,
            keys: hashmap![VirtualKeyCode::E => KeyPress::new(100, 500), VirtualKeyCode::I => KeyPress::new(100, 2000), VirtualKeyCode::Escape => KeyPress::new(100, 100)],
        }
    }
}

pub struct State {
    ecs: World,
    runstate: RunState,
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
        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        // if self.runstate == RunState::Running {
        //     self.run_systems();
        //     self.runstate = RunState::Paused;
        // } else {
        player_input(self);
        self.run_systems();
        //self.runstate = RunState::Running;
        //self.runstate = player_input(self, ctx);
        //}

        damage_system::delete_the_dead(&mut self.ecs);

        draw_map(&self.ecs, ctx);

        // Renderables
        {
            let positions = self.ecs.read_storage::<Position>();
            let renderables = self.ecs.read_storage::<Renderable>();
            let map = self.ecs.fetch::<Map>();

            for (pos, render) in (&positions, &renderables).join() {
                let idx = map.xy_idx(pos.x, pos.y);
                if map.visible_tiles[idx] {
                    ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
                }
            }
        }

        gui::draw_ui(self, ctx);
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    Paused,
    Running,
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
    world.register::<Potion>();
    world.register::<WantsToPickupItem>();
    world.register::<InBackpack>();

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

    // GameState
    let gs = State {
        ecs: world,
        runstate: RunState::Running,
        client: Client::default(),
    };
    rltk::main_loop(context, gs)
}
