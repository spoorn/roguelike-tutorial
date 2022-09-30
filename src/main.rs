#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::SystemTime;
use rltk::{BResult, GameState, Rltk, RltkBuilder, RGB, RandomNumberGenerator, VirtualKeyCode};
use specs::{Builder, Join, RunNow, World, WorldExt};

use crate::components::{Monster, Player, Position, Renderable, Viewshed};
use crate::map::{draw_map, Map, TileType};
use crate::monster_ai_system::MonsterAI;
use crate::player::{player_input, player_input_free_movement};
use crate::visibility_system::VisibilitySystem;

mod components;
mod map;
mod player;
mod rect;
mod visibility_system;
mod monster_ai_system;

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
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum RunState { Paused, Running }

fn main() -> BResult<()> {
    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build()?;
    
    // World
    let mut world = World::new();
    world.register::<Position>();
    world.register::<Renderable>();
    world.register::<Player>();
    world.register::<Viewshed>();
    world.register::<Monster>();
    
    let map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();
    
    // Player
    world
        .create_entity()
        .with(Position {
            x: player_x,
            y: player_y,
        })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player{})
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .build();
    
    // Monsters
    let mut rng = RandomNumberGenerator::new();
    for room in map.rooms.iter().skip(1) {
        let (x, y) = room.center();
        
        let glyph;
        let roll = rng.roll_dice(1, 2);
        match roll {
            1 => { glyph = rltk::to_cp437('g') }
            _ => { glyph = rltk::to_cp437('o') }
        }
        
        world.create_entity()
            .with(Position { x, y })
            .with(Renderable {
                glyph,
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK),
            })
            .with(Viewshed {
                visible_tiles: Vec::new(),
                range: 8,
                dirty: true,
            })
            .with(Monster{})
            .build();
    }
    
    world.insert(map);
    
    // GameState
    let gs = State { ecs: world, runstate: RunState::Running, client: Client::default() };
    rltk::main_loop(context, gs)
}
