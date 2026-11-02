//! Map generation with libd2 (https://github.com/jaenster/libd2).
//!
//! This replaces the old d2-mapgen.exe + Diablo II 1.13c setup. libd2 is a clean-room rebuild of
//! the Diablo II level generator with the game tables compiled in, so it needs no game files.
//!
//! The output is shaped exactly like the JSON d2-mapgen used to print, so everything downstream
//! (map images, POIs, walkable exits, pathfinding) keeps working unchanged:
//!   * offset / size are in subtiles (world coordinates)
//!   * object x / y are level-local subtiles
//!   * `map` is one run-length list per row: blocked, walkable, blocked, walkable, ...
//!     always starting with a blocked run and always ending on a walkable run.

use std::cell::RefCell;
use std::time::Instant;

use libd2::drlg::{Collision, CollisionGrid, Difficulty, Generator, LevelRef, Preset};

use super::jsondata::{LevelData, Object, Offset, Size, SeedData};

/// Level IDs 1..=136 are the original game. Anything above that is a mod's custom level and can
/// only be mapped live from game memory (see `crate::livemap`).
pub const LAST_VANILLA_LEVEL: u32 = 136;

thread_local! {
    // Loading the game tables is the expensive part, so keep one generator per thread.
    // The generator is Send but not Sync, which is exactly what thread_local gives us.
    static GENERATOR: RefCell<Option<Generator>> = const { RefCell::new(None) };
}

pub fn get_seed_data(map_seed: u32, difficulty: u32) -> SeedData {
    let start = Instant::now();
    let mut levels: Vec<LevelData> = Vec::new();

    GENERATOR.with(|cell| {
        let mut slot = cell.borrow_mut();
        if slot.is_none() {
            match Generator::new() {
                Ok(g) => *slot = Some(g),
                Err(e) => {
                    log::error!("libd2: could not start the map generator: {:?}", e);
                    return;
                }
            }
        }
        let drlg = slot.as_ref().unwrap();
        let diff = Difficulty::try_from(difficulty as i32).unwrap_or(Difficulty::Normal);

        for act in 0u8..5 {
            let act_ref = match drlg.act(map_seed, act).difficulty(diff).open() {
                Ok(a) => a,
                Err(e) => {
                    log::error!("libd2: failed to generate act {} for seed {}: {:?}", act + 1, map_seed, e);
                    continue;
                }
            };
            for level in act_ref.levels() {
                if !level.placed() {
                    continue;
                }
                match convert_level(&level) {
                    Some(level_data) => levels.push(level_data),
                    None => log::debug!("libd2: skipped level {}", level.id().0),
                }
            }
        }
    });

    levels.sort_by_key(|l| l.id);
    log::info!(
        "libd2 generated {} levels for seed {} difficulty {} in {:.3} seconds",
        levels.len(),
        map_seed,
        difficulty,
        start.elapsed().as_secs_f64()
    );

    SeedData { seed: map_seed, difficulty, levels }
}

fn convert_level(level: &LevelRef<'_>) -> Option<LevelData> {
    let id = level.id().0;
    if id <= 0 {
        return None;
    }
    let origin = level.origin().ok()?;
    let grid = level.collision().ok()?;
    let width = grid.width();
    let height = grid.height();
    if width == 0 || height == 0 || origin.x < 0 || origin.y < 0 {
        return None;
    }

    let map = encode_grid(&grid);
    let objects = convert_presets(level);

    Some(LevelData {
        level_type: "map".to_owned(),
        id: id as u32,
        name: Default::default(),
        offset: Offset {
            x: (origin.x * libd2::drlg::SUBTILES) as u32,
            y: (origin.y * libd2::drlg::SUBTILES) as u32,
        },
        size: Size { width: width as u32, height: height as u32 },
        objects,
        map,
        level_image: Default::default(),
    })
}

/// Walls (and void, which has every bit set) are blocked; everything else is floor.
fn is_blocked(c: Collision) -> bool {
    c.intersects(Collision::WALL)
}

fn encode_grid(grid: &CollisionGrid) -> Vec<Vec<u64>> {
    grid.rows()
        .map(|row| encode_row(row.iter().map(|c| is_blocked(*c))))
        .collect()
}

/// Run-length encode one row in d2-mapgen's format. `blocked` yields true for blocked cells.
pub fn encode_row(blocked: impl Iterator<Item = bool>) -> Vec<u64> {
    let mut runs: Vec<u64> = Vec::new();
    let mut current_blocked = true; // the first run is always a blocked run (may be 0 long)
    let mut count: u64 = 0;
    let mut any_walkable = false;
    for b in blocked {
        if b == current_blocked {
            count += 1;
        } else {
            runs.push(count);
            current_blocked = b;
            count = 1;
        }
        if !b {
            any_walkable = true;
        }
    }
    if !any_walkable {
        // An empty row means "all blocked".
        return Vec::new();
    }
    runs.push(count);
    // The reader treats a trailing blocked run as "rest of the row is walkable",
    // so always finish on a walkable run, even if it is zero long.
    if runs.len() % 2 == 1 {
        runs.push(0);
    }
    runs
}

fn convert_presets(level: &LevelRef<'_>) -> Vec<Object> {
    let presets = level.presets().unwrap_or_default();
    let adjacents = level.adjacents().unwrap_or_default();
    let mut objects = Vec::with_capacity(presets.len());

    for p in presets {
        if p.at.x < 0 || p.at.y < 0 {
            continue;
        }
        let x = p.at.x as u32;
        let y = p.at.y as u32;
        match p.what {
            Preset::Object(obj_id) => {
                let name = libd2::drlg::object_name(obj_id);
                let class = match obj_id.0 {
                    580 | 581 => "chest-super",
                    _ => "",
                };
                objects.push(Object {
                    id: obj_id.0 as u32,
                    object_type: "object".to_owned(),
                    x,
                    y,
                    name,
                    class: class.to_owned(),
                    op: 0,
                    is_good_exit: false,
                    owned_level_id: 0,
                });
            }
            Preset::Monster(mon_id) => {
                objects.push(Object::new_npc(x, y, mon_id.0 as u32));
            }
            Preset::Exit(_warp) => {
                // A warp (stairs, cave mouth, doorway). Its destination is the adjacent level
                // whose crossing point is closest to the warp.
                let dest = adjacents
                    .iter()
                    .min_by_key(|a| (a.at.x - p.at.x).abs() + (a.at.y - p.at.y).abs())
                    .map(|a| a.to.0)
                    .unwrap_or(0);
                if dest <= 0 {
                    continue;
                }
                objects.push(Object {
                    id: dest as u32,
                    object_type: "exit".to_owned(),
                    x,
                    y,
                    name: String::new(),
                    class: String::new(),
                    op: 0,
                    is_good_exit: false,
                    owned_level_id: 0,
                });
            }
            Preset::Other { .. } => {}
        }
    }
    objects
}

#[cfg(test)]
mod tests {
    use super::encode_row;

    fn decode(runs: &[u64], width: usize) -> Vec<bool> {
        // mirrors mapgrid::level_data_to_walkable for one row (true = walkable)
        let mut row = vec![false; width];
        let mut fill = false;
        let mut x = 0usize;
        let last = runs.len().wrapping_sub(1);
        for (i, r) in runs.iter().enumerate() {
            fill = !fill;
            if !fill {
                for n in 0..*r as usize {
                    row[n + x] = true;
                }
            }
            x += *r as usize;
            if i == last && fill {
                let extra = width.wrapping_sub(x).wrapping_sub(1);
                for n in 0..extra {
                    row[n + x] = true;
                }
            }
        }
        row
    }

    #[test]
    fn round_trip() {
        let rows: Vec<Vec<bool>> = vec![
            vec![true, true, false, false, true],
            vec![false, false, true],
            vec![true, true, true],
            vec![false, false, false],
            vec![true, false, true, false, true, true],
        ];
        for blocked in rows {
            let runs = encode_row(blocked.iter().copied());
            let walk = decode(&runs, blocked.len());
            let expected: Vec<bool> = blocked.iter().map(|b| !b).collect();
            assert_eq!(walk, expected, "runs {:?}", runs);
        }
    }
}
