#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::SystemTime;

use bounded_vec_deque::BoundedVecDeque;
use rltk::{BResult, GameState, Point, RandomNumberGenerator, Rltk, RltkBuilder, VirtualKeyCode};
use specs::{Builder, Join, RunNow, World, WorldExt};

use crate::components::{BlocksTile, CombatStats, Monster, MovementSpeed, Name, Player, Position, Renderable, SufferDamage, Viewshed, WantsToMelee};
use crate::damage_system::DamageSystem;
use crate::gamelog::GameLog;
use crate::map::{draw_map, Map};
use crate::map_indexing_system::MapIndexingSystem;
use crate::melee_combat_system::MeleeCombatSystem;
use crate::monster_ai_system::MonsterAI;
use crate::player::player_input_free_movement;
use crate::visibility_system::VisibilitySystem;

mod components;
mod map;
mod map_indexing_system;
mod player;
mod rect;
mod visibility_system;
mod monster_ai_system;
mod movement_util;
mod melee_combat_system;
mod damage_system;
mod gui;
mod gamelog;
mod spawner;

#[derive(Debug, Default)]
pub struct Client {
    pub last_key_pressed: Option<VirtualKeyCode>,
    pub last_key_time: Option<SystemTime>
}

pub struct State {
    ecs: World,
    runstate: RunState,
    client: Client
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);
        let mut mob = MonsterAI {};
        mob.run_now(&self.ecs);
        let mut mapindex = MapIndexingSystem{};
        mapindex.run_now(&self.ecs);
        let mut melee_combat = MeleeCombatSystem{};
        melee_combat.run_now(&self.ecs);
        let mut damage = DamageSystem{};
        damage.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        if self.runstate == RunState::Running {
            self.run_systems();
            self.runstate = RunState::Paused;
        } else {
            player_input_free_movement(self);
            self.runstate = RunState::Running;
            //self.runstate = player_input(self, ctx);
        }
        
        damage_system::delete_the_dead(&mut self.ecs);

        draw_map(&self.ecs, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let map = self.ecs.fetch::<Map>();

        for (pos, render) in (&positions, &renderables).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
            }
        }
        
        gui::draw_ui(&self.ecs, ctx);
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum RunState { Paused, Running }

fn main() -> BResult<()> {
    let mut context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build()?;
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
        let (x, y) = room.center();
        
        spawner::random_monster(&mut world, x, y);
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
    let gs = State { ecs: world, runstate: RunState::Running, client: Client::default() };
    rltk::main_loop(context, gs)
}
