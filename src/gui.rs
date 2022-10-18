use rltk::{Point, RGB, Rltk};
use specs::{Entity, Join, WorldExt};

use crate::{CombatStats, Consumable, GameLog, InBackpack, Map, Name, Player, Position, State, WantsToDropItem, WantsToUseItem, World};

pub fn draw_ui(gs: &mut State, ctx: &mut Rltk) {
    {
        let ecs = &mut gs.ecs;
        ctx.draw_box(0, 43, 79, 6, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));

        let combat_stats = ecs.read_storage::<CombatStats>();
        let players = ecs.read_storage::<Player>();
        for (_player, stats) in (&players, &combat_stats).join() {
            let health = format!(" HP: {} / {}", stats.hp, stats.max_hp);
            ctx.print_color(12, 43, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), &health);

            ctx.draw_bar_horizontal(28, 43, 51, stats.hp, stats.max_hp, RGB::named(rltk::RED), RGB::named(rltk::BLACK));
        }

        let log = ecs.fetch::<GameLog>();
        let mut y = 48;
        for s in log.entries.iter().rev() {
            if y >= 44 {
                ctx.print(2, y, s);
            } else {
                break;
            }
            y -= 1;
        }

        // Mouse tooltip
        let mouse_pos = ctx.mouse_pos();
        ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::PINK));

        draw_tooltips(ecs, ctx);
    }

    show_inventory(gs, ctx);
}

pub fn show_inventory(gs: &mut State, ctx: &mut Rltk) {
    if gs.client.show_inventory {
        let player_entity = gs.ecs.fetch::<Entity>();
        let names = gs.ecs.read_storage::<Name>();
        let backpack = gs.ecs.read_storage::<InBackpack>();
        let entities = gs.ecs.entities();

        let inventory = (&backpack, &names).join().filter(|item| item.0.owner == *player_entity);
        let count = inventory.count();

        let mut y = (25 - (count / 2)) as i32;
        ctx.draw_box(15, y - 2, 31, (count + 3) as i32, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));
        if gs.client.drop_inventory {
            ctx.print_color(18, y - 2, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Drop Which Item?");
        } else {
            ctx.print_color(18, y - 2, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Inventory");
        }
        ctx.print_color(
            18,
            y + count as i32 + 1,
            RGB::named(rltk::YELLOW),
            RGB::named(rltk::BLACK),
            "ESCAPE to close/cancel",
        );

        let mut equippable: Vec<Entity> = Vec::new();
        let mut j = 0;
        for (entity, _pack, name) in (&entities, &backpack, &names).join().filter(|item| item.1.owner == *player_entity)
        {
            ctx.set(17, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437('('));
            ctx.set(18, y, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), 97 + j as rltk::FontCharType);
            ctx.set(19, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437(')'));

            ctx.print(21, y, &name.name.to_string());
            equippable.push(entity);
            y += 1;
            j += 1;
        }

        match ctx.key {
            Some(key) => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < count as i32 {
                    let selection = selection as usize;

                    if gs.client.drop_inventory {
                        // Drop items
                        let mut drop_intents = gs.ecs.write_storage::<WantsToDropItem>();
                        drop_intents
                            .insert(
                                *player_entity,
                                WantsToDropItem {
                                    item: equippable[selection],
                                },
                            )
                            .expect("Unable to insert item to drop");
                    } else {
                        // Use Items
                        let consumables = gs.ecs.read_storage::<Consumable>();
                        if consumables.contains(equippable[selection]) {
                            let mut use_item = gs.ecs.write_storage::<WantsToUseItem>();
                            use_item
                                .insert(
                                    *player_entity,
                                    WantsToUseItem {
                                        item: equippable[selection],
                                    },
                                )
                                .expect("Unable to insert drink potion intent");
                        }
                    }
                }
            }
            None => {}
        }
    }
}

fn draw_tooltips(ecs: &World, ctx: &mut Rltk) {
    let map = ecs.fetch::<Map>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();

    let (mouse_x, mouse_y) = ctx.mouse_pos();
    if mouse_x >= map.width || mouse_y >= map.height {
        return;
    }

    let mut tooltip: Vec<String> = Vec::new();
    for (name, position) in (&names, &positions).join() {
        let idx = map.xy_idx(position.x, position.y);
        if position.x == mouse_x && position.y == mouse_y && map.visible_tiles[idx] {
            tooltip.push(name.name.to_string());
        }
    }

    if !tooltip.is_empty() {
        let mut width: i32 = 0;
        for s in tooltip.iter() {
            if width < s.len() as i32 {
                width = s.len() as i32;
            }
        }
        width += 3;

        if mouse_x > 40 {
            let arrow_pos = Point::new(mouse_x - 2, mouse_y);
            let left_x = mouse_x - width;
            let mut y = mouse_y;
            for s in tooltip.iter() {
                ctx.print_color(left_x, y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), s);
                let padding = (width - s.len() as i32) - 1;
                for i in 0..padding {
                    ctx.print_color(
                        arrow_pos.x - i,
                        y,
                        RGB::named(rltk::WHITE),
                        RGB::named(rltk::GREY),
                        &" ".to_string(),
                    );
                }
                y += 1;
            }
            ctx.print_color(
                arrow_pos.x,
                arrow_pos.y,
                RGB::named(rltk::WHITE),
                RGB::named(rltk::GREY),
                &"->".to_string(),
            );
        } else {
            let arrow_pos = Point::new(mouse_x + 1, mouse_y);
            let left_x = mouse_x + 3;
            let mut y = mouse_y;
            for s in tooltip.iter() {
                ctx.print_color(left_x + 1, y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), s);
                let padding = (width - s.len() as i32) - 1;
                for i in 0..padding {
                    ctx.print_color(
                        arrow_pos.x + 1 + i,
                        y,
                        RGB::named(rltk::WHITE),
                        RGB::named(rltk::GREY),
                        &" ".to_string(),
                    );
                }
                y += 1;
            }
            ctx.print_color(
                arrow_pos.x,
                arrow_pos.y,
                RGB::named(rltk::WHITE),
                RGB::named(rltk::GREY),
                &"<-".to_string(),
            );
        }
    }
}
