use notan::draw::*;
use notan::prelude::*;

use crate::mapgeneration::jsondata::LevelData;
use crate::mapgeneration::jsondata::LevelName;
use crate::mapgeneration::mapgrid::MapGrid;
use crate::mapgeneration::pathfind;
use crate::mapgeneration::pathfind::Pos;
use crate::mapgeneration::pois::POI;
use crate::mapgeneration::pois::POIType;
use crate::memory::gamedata::GameData;
use crate::settings::Settings;
use num_traits::FromPrimitive;


use super::draw_lines::get_next_exits;

pub fn draw_pathfinding(draw: &mut Draw, game_data: &GameData, settings: &Settings, this_level: &mut LevelData, map_position_x: f32, map_position_y: f32, player_pos_x: f32, player_pos_y: f32) {
    if game_data.is_in_town() {
        return;
    }
    let mapgrid = match &this_level.level_image.map_grid {
        Some(map_grid) => map_grid,
        None => return,
    };

    let player_pos: Pos = Pos(game_data.player.pos_x as i16 - this_level.offset.x as i16, game_data.player.pos_y as i16 - this_level.offset.y as i16);

    // draw paths to exits
    if settings.lines.exit_path_enabled {
        let mut end_positions = get_exit_endpoints(LevelName::from_u32(this_level.id).unwrap_or_default(), &this_level);
        // Canyon of the Magi: the real Tal Rasha's tomb
        for poi in this_level.level_image.pois.iter().filter(|p| p.poi_type == POIType::GoodExit) {
            end_positions.push((poi.pos_x as u32, poi.pos_y as u32));
        }
        let exit_path_color: Color = convert_color(settings.lines.exit_rgba);
        for exit_pos in end_positions.iter() {
            let end_pos: Pos = Pos(exit_pos.0 as i16, exit_pos.1 as i16);
            draw_path(draw, mapgrid, exit_path_color, player_pos, end_pos, map_position_x, map_position_y, player_pos_x, player_pos_y, settings.lines.line_width);
        }
    }

    // draw path to waypoint
    if settings.lines.waypoint_path_enabled {
        match get_waypoint(&this_level) {
            Some(waypoint) => {
                let waypoint_line_color: Color = convert_color(settings.lines.waypoint_rgba);
                let end_pos: Pos = Pos(waypoint.0 as i16, waypoint.1 as i16);
                draw_path(draw, mapgrid, waypoint_line_color, player_pos, end_pos, map_position_x, map_position_y, player_pos_x, player_pos_y, settings.lines.line_width);
            },
            None => (),
        };
    }

    // draw path to npc spawn
    if settings.lines.boss_path_enabled {
        let boss_line_color: Color = convert_color(settings.lines.boss_rgba);
        for npc_spawn in get_boss_spawns(&this_level) {
            let end_pos: Pos = Pos(npc_spawn.0 as i16, npc_spawn.1 as i16);
            draw_path(draw, mapgrid, boss_line_color, player_pos, end_pos, map_position_x, map_position_y, player_pos_x, player_pos_y, settings.lines.line_width);
        }
    }

    // draw paths to quest items
    if settings.lines.quest_path_enabled {
        let end_positions = get_quest_items( &this_level);
        let quest_path_color: Color = convert_color(settings.lines.quest_rgba);
        for quest_pos in end_positions.iter() {
            let end_pos: Pos = if this_level.name == LevelName::DarkWood {
                Pos(quest_pos.0 as i16 + 5, quest_pos.1 as i16)
            } else {
                Pos(quest_pos.0 as i16, quest_pos.1 as i16)
            };
            draw_path(draw, mapgrid, quest_path_color, player_pos, end_pos, map_position_x, map_position_y, player_pos_x, player_pos_y, settings.lines.line_width);
        }
    }

}

pub fn draw_path(draw: &mut Draw, mapgrid: &MapGrid, path_color: Color, player_pos: Pos, end_pos: Pos, map_position_x: f32, map_position_y: f32, player_pos_x: f32, player_pos_y: f32, line_width: f32) {
    let mut path_data: Vec<Pos> = cached_path(mapgrid, player_pos, end_pos);
    if path_data.is_empty() {
        // no walkable route known (e.g. a live map not explored yet): straight line instead
        path_data = vec![player_pos, end_pos];
    }
    let path_start = path_data.get(0).unwrap();
    let mut path = draw.path();
    path.move_to(path_start.0.into(), path_start.1.into());
        
    for pos in path_data.iter() {
        path.line_to(pos.0.into(), pos.1.into());
    }

    path.translate(map_position_x, map_position_y)
        .rotate_degrees_from((map_position_x + player_pos_x, map_position_y + player_pos_y), 45.0)
        .round_join()
        .color(path_color)
        .stroke(line_width / 2.0)
        .alpha(0.5);
}


pub fn get_exit_endpoints(current_level: LevelName, this_level: &LevelData) -> Vec<(u32, u32)> {
    let mut exit_positions: Vec<(u32, u32)> = vec![];
    
    let exits_vec = get_next_exits(current_level);
    for exit in exits_vec.iter() {
        for poi in this_level.level_image.pois.iter() {
            if poi.poi_type == POIType::Exit {
                if poi.id == *exit as u32 {
                    exit_positions.push((poi.pos_x as u32, poi.pos_y as u32));
                }
            } 
        }
    }
    exit_positions
}

pub fn get_waypoint(this_level: &LevelData) -> Option<(u32, u32)> {
    match this_level.level_image.pois.iter().find(|w| w.poi_type == POIType::Waypoint) {
        Some(wp) => Some((wp.pos_x as u32, wp.pos_y as u32)),
        None => None,
    }
}

pub fn get_boss_spawns(this_level: &LevelData) -> Vec<(u32, u32)> {
    this_level
        .level_image
        .pois
        .iter()
        .filter(|w| w.poi_type == POIType::NPCSpawn)
        .map(|b| (b.pos_x as u32, b.pos_y as u32))
        .collect()
}

#[allow(dead_code)]
pub fn get_boss_spawn(this_level: &LevelData) -> Option<(u32, u32)> {
    match this_level.level_image.pois.iter().find(|w| w.poi_type == POIType::NPCSpawn) {
        Some(boss) => Some((boss.pos_x as u32, boss.pos_y as u32)),
        None => None,
    }
}

pub fn get_quest_items(this_level: &LevelData) -> Vec<(u32, u32)> {
    let quest_pois: Vec<&POI> = this_level.level_image.pois.iter().filter(|w| w.poi_type == POIType::QuestItem).collect();
    let mut quest_items: Vec<(u32, u32)> = vec![];
    quest_pois.iter().for_each(|poi| quest_items.push((poi.pos_x as u32, poi.pos_y as u32)));
    quest_items
}

fn convert_color(color_arr: [u8; 4]) -> Color {
    Color::from_bytes(color_arr[0], color_arr[1], color_arr[2], color_arr[3])
}


/// Nearest walkable cell to `p` (markers often sit on a wall or inside an object).
fn nearest_walkable(grid: &MapGrid, p: Pos, radius: i16) -> Option<Pos> {
    let ok = |x: i16, y: i16| x >= 0 && y >= 0 && x < grid.width && y < grid.height && grid.tiles[y as usize][x as usize];
    if ok(p.0, p.1) {
        return Some(p);
    }
    for r in 1..=radius {
        for d in -r..=r {
            for (x, y) in [(p.0 + d, p.1 - r), (p.0 + d, p.1 + r), (p.0 - r, p.1 + d), (p.0 + r, p.1 + d)] {
                if ok(x, y) {
                    return Some(Pos(x, y));
                }
            }
        }
    }
    None
}

thread_local! {
    // (grid identity, target) -> (start it was computed from, when, path)
    static PATHS: std::cell::RefCell<std::collections::HashMap<(usize, Pos), (Pos, std::time::Instant, Vec<Pos>)>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

/// A* path from the player to a target, recomputed only when the player has moved a few
/// steps or a second has passed - a big dungeon is too large to search every frame.
fn cached_path(grid: &MapGrid, start: Pos, end: Pos) -> Vec<Pos> {
    let key = (grid as *const MapGrid as usize, end);
    let fresh = PATHS.with(|c| {
        c.borrow().get(&key).and_then(|(from, when, path)| {
            let moved = (from.0 - start.0).abs() + (from.1 - start.1).abs();
            if moved <= 3 && when.elapsed() < std::time::Duration::from_secs(1) {
                Some(path.clone())
            } else {
                None
            }
        })
    });
    if let Some(path) = fresh {
        return path;
    }
    let path = match (nearest_walkable(grid, start, 8), nearest_walkable(grid, end, 30)) {
        (Some(s), Some(e)) => pathfind::get_path_data(grid, s, e),
        _ => vec![],
    };
    PATHS.with(|c| {
        let mut c = c.borrow_mut();
        if c.len() > 64 {
            c.clear();
        }
        c.insert(key, (start, std::time::Instant::now(), path.clone()));
    });
    path
}
