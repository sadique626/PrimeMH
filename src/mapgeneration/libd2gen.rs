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

use libd2::drlg::{Collision, Difficulty, LevelKind, Generator, LevelRef, Preset};

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
                // interior levels (dungeons, tombs, cellars...) are not "placed" on the
                // surface but still have their own coordinates, so keep every level
                match convert_level(&level, map_seed) {
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

fn convert_level(level: &LevelRef<'_>, map_seed: u32) -> Option<LevelData> {
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

    // Dungeon levels reserve a huge area (e.g. 1000x1000 subtiles) but only use part of it.
    // Crop to the floor that actually exists (plus a small border) so images stay small.
    let cells = grid.cells();
    let (mut x0, mut y0, mut x1, mut y1) = (width, height, 0usize, 0usize);
    for y in 0..height {
        for x in 0..width {
            if !is_blocked(cells[y * width + x]) {
                x0 = x0.min(x);
                y0 = y0.min(y);
                x1 = x1.max(x);
                y1 = y1.max(y);
            }
        }
    }
    if x1 < x0 || y1 < y0 {
        return None; // no floor at all
    }
    // objects first, in uncropped level coordinates
    let walk = Walk { cells, width, height };
    let all_objects: Vec<Object> = convert_presets(level, map_seed, &walk);

    let (x0, y0, x1, y1) = if matches!(level.kind(), LevelKind::Maze) {
        // keep every exit inside the cropped area: a warp's marker sits at its room's centre,
        // which can lie outside the floor's bounding box
        let (mut x0, mut y0, mut x1, mut y1) = (x0, y0, x1, y1);
        for o in all_objects.iter().filter(|o| o.object_type == "exit") {
            x0 = x0.min(o.x as usize);
            y0 = y0.min(o.y as usize);
            x1 = x1.max(o.x as usize);
            y1 = y1.max(o.y as usize);
        }
        (x0.saturating_sub(2), y0.saturating_sub(2), (x1 + 2).min(width - 1), (y1 + 2).min(height - 1))
    } else {
        // only random dungeons are cropped; fixed layouts keep their original coordinates
        // (PrimeMH has hard-coded positions for some of them, e.g. Nihlathak in Halls of Vaught)
        (0, 0, width - 1, height - 1)
    };
    let (cw, ch) = (x1 - x0 + 1, y1 - y0 + 1);

    let map: Vec<Vec<u64>> = (y0..=y1)
        .map(|y| encode_row((x0..=x1).map(|x| is_blocked(cells[y * width + x]))))
        .collect();
    let objects: Vec<Object> = all_objects
        .into_iter()
        .filter(|o| (o.x as usize) >= x0 && (o.x as usize) <= x1 && (o.y as usize) >= y0 && (o.y as usize) <= y1)
        .map(|mut o| {
            o.x -= x0 as u32;
            o.y -= y0 as u32;
            o
        })
        .collect();

    Some(LevelData {
        level_type: "map".to_owned(),
        id: id as u32,
        name: Default::default(),
        offset: Offset {
            x: (origin.x * libd2::drlg::SUBTILES) as u32 + x0 as u32,
            y: (origin.y * libd2::drlg::SUBTILES) as u32 + y0 as u32,
        },
        size: Size { width: cw as u32, height: ch as u32 },
        objects,
        map,
        level_image: Default::default(),
    })
}

/// Walls (and void, which has every bit set) are blocked; everything else is floor.
fn is_blocked(c: Collision) -> bool {
    c.intersects(Collision::WALL)
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

/// The Horadric Staff ("true") tomb in Act II, rolled from the map seed exactly like the game
/// does (DRLG_AllocDrlgActMisc): returns a level id between 66 and 72.
pub fn staff_tomb_level(map_seed: u32) -> u32 {
    const MUL: u64 = 0x6AC6_90C5;
    let next = |(low, high): (u32, u32)| -> (u32, u32) {
        let state = (low as u64).wrapping_mul(MUL).wrapping_add(high as u64);
        (state as u32, (state >> 32) as u32)
    };
    let mut s = next((map_seed, 0x29A));
    loop {
        let prod = next(s);
        let staff = prod.0 % 7;
        s = next(prod);
        let boss = s.0 % 7;
        if staff != boss {
            return staff + 66;
        }
    }
}

/// Objects.txt name for an object id. libd2 looks names up by file row, but the file has an
/// "Expansion" divider row before id 410 that the game skips, so from there on the row is one
/// further down than the id. Without this, e.g. the Halls of Pain waypoint (496) reads as
/// "magic shrine".
fn object_name(id: i32) -> String {
    let row = if id >= 410 { id + 1 } else { id };
    libd2::drlg::object_name(libd2::drlg::ObjectId(row))
}

fn dist(ax: i32, ay: i32, bx: i32, by: i32) -> i32 {
    (ax - bx).abs() + (ay - by).abs()
}

fn convert_presets(level: &LevelRef<'_>, map_seed: u32, walk: &Walk) -> Vec<Object> {
    let presets = level.presets().unwrap_or_default();
    let adjacents = level.adjacents().unwrap_or_default();
    let level_id = level.id().0;
    let mut objects = Vec::with_capacity(presets.len() + adjacents.len());
    let mut warp_spots: Vec<(i32, i32)> = Vec::new();

    for p in presets {
        if p.at.x < 0 || p.at.y < 0 {
            continue;
        }
        let x = p.at.x as u32;
        let y = p.at.y as u32;
        match p.what {
            Preset::Object(obj_id) => {
                let name = object_name(obj_id.0);
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
            Preset::Exit(_) => warp_spots.push((p.at.x, p.at.y)),
            Preset::Other { .. } => {}
        }
    }

    // Zone transitions. libd2 lists every level this one connects to (stairs, cave mouths,
    // doorways and walk-across borders). Its position is the centre of the room holding the
    // crossing; if a warp tile sits in that room, use the warp's exact spot instead.
    let staff_tomb = staff_tomb_level(map_seed) as i32;

    // Group the crossing points per destination. A walk-across border is reported once per
    // border room (every 40 subtiles), so chain points that sit close together into one
    // crossing and put a single label in its middle. Separate entrances stay separate.
    let mut clusters: Vec<(i32, Vec<(i32, i32)>)> = Vec::new(); // (dest, points)
    for adj in &adjacents {
        let dest = adj.to.0;
        if dest <= 0 || dest == level_id || adj.at.x < 0 || adj.at.y < 0 {
            continue;
        }
        let pt = (adj.at.x, adj.at.y);
        let touching: Vec<usize> = clusters
            .iter()
            .enumerate()
            .filter(|(_, (d, pts))| *d == dest && pts.iter().any(|q| dist(q.0, q.1, pt.0, pt.1) <= 45))
            .map(|(i, _)| i)
            .collect();
        match touching.first() {
            None => clusters.push((dest, vec![pt])),
            Some(&first) => {
                clusters[first].1.push(pt);
                // merge any other clusters this point bridges
                for &i in touching.iter().skip(1).rev() {
                    let moved = clusters.remove(i).1;
                    clusters[first].1.extend(moved);
                }
            }
        }
    }

    for (dest, pts) in clusters {
        let n = pts.len() as i32;
        let mx = pts.iter().map(|p| p.0).sum::<i32>() / n;
        let my = pts.iter().map(|p| p.1).sum::<i32>() / n;
        let is_good_exit = level_id == 46 && dest == staff_tomb;

        // Walk-across border: libd2 reports every pair of rooms that touch across the seam,
        // not just the ones you can walk through, so the middle of those rooms can be far from
        // the real gap. Find the open stretch(es) of floor on that edge of the map instead.
        if let Some(edge) = walk.border_edge(&pts) {
            let crossings = walk.crossings(edge, &pts);
            for (x, y) in crossings {
                objects.push(exit_object(dest, x, y, "walkable", is_good_exit, level_id));
            }
            continue;
        }

        // Warp (stairs, cave mouth, door): the room centre, or a warp tile near it if libd2 has
        // one. PrimeMH moves the marker onto the real warp once the game has it loaded.
        let (mut x, mut y) = (mx, my);
        if let Some(&(wx, wy)) = warp_spots.iter().filter(|w| dist(w.0, w.1, x, y) <= 30).min_by_key(|w| dist(w.0, w.1, x, y)) {
            x = wx;
            y = wy;
        }
        objects.push(exit_object(dest, x, y, "", is_good_exit, level_id));
    }
    objects
}

fn exit_object(dest: i32, x: i32, y: i32, class: &str, is_good_exit: bool, level_id: i32) -> Object {
    Object {
        id: dest as u32,
        object_type: "exit".to_owned(),
        x: x.max(0) as u32,
        y: y.max(0) as u32,
        name: String::new(),
        class: class.to_owned(),
        op: 0,
        is_good_exit,
        owned_level_id: level_id as u32,
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Edge {
    Left,
    Right,
    Top,
    Bottom,
}

/// The level's collision, for finding where its borders are open.
struct Walk<'a> {
    cells: &'a [Collision],
    width: usize,
    height: usize,
}

impl Walk<'_> {
    fn open(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return false;
        }
        !is_blocked(self.cells[y as usize * self.width + x as usize])
    }

    /// If these crossing points all hug one edge of the level, that edge (a walk-across border).
    fn border_edge(&self, pts: &[(i32, i32)]) -> Option<Edge> {
        const NEAR: i32 = 30; // border rooms are 40 subtiles, so their centre sits ~20 in
        let (w, h) = (self.width as i32, self.height as i32);
        let all = |f: &dyn Fn(&(i32, i32)) -> bool| pts.iter().all(f);
        if all(&|p| p.0 <= NEAR) {
            Some(Edge::Left)
        } else if all(&|p| p.0 >= w - 1 - NEAR) {
            Some(Edge::Right)
        } else if all(&|p| p.1 <= NEAR) {
            Some(Edge::Top)
        } else if all(&|p| p.1 >= h - 1 - NEAR) {
            Some(Edge::Bottom)
        } else {
            None
        }
    }

    /// Centres of the open stretches along `edge`, within the span of the touching rooms.
    fn crossings(&self, edge: Edge, pts: &[(i32, i32)]) -> Vec<(i32, i32)> {
        let (w, h) = (self.width as i32, self.height as i32);
        let along: Vec<i32> = pts.iter().map(|p| if matches!(edge, Edge::Left | Edge::Right) { p.1 } else { p.0 }).collect();
        let limit = if matches!(edge, Edge::Left | Edge::Right) { h } else { w };
        let lo = (along.iter().min().copied().unwrap_or(0) - 25).max(0);
        let hi = (along.iter().max().copied().unwrap_or(0) + 25).min(limit - 1);
        // a cell on the edge counts as open if any of the outermost 3 rows/columns is floor there
        let open_at = |t: i32| -> bool {
            (0..3).any(|d| match edge {
                Edge::Left => self.open(d, t),
                Edge::Right => self.open(w - 1 - d, t),
                Edge::Top => self.open(t, d),
                Edge::Bottom => self.open(t, h - 1 - d),
            })
        };
        let mut runs: Vec<(i32, i32)> = Vec::new(); // (start, end) inclusive
        let mut t = lo;
        while t <= hi {
            if open_at(t) {
                let start = t;
                while t <= hi && open_at(t) {
                    t += 1;
                }
                // join stretches split by a stray tree or rock
                match runs.last_mut() {
                    Some(last) if start - last.1 <= 8 => last.1 = t - 1,
                    _ => runs.push((start, t - 1)),
                }
            }
            t += 1;
        }
        runs.into_iter()
            .filter(|(a, b)| b - a + 1 >= 3)
            .map(|(a, b)| {
                let mid = (a + b) / 2;
                match edge {
                    Edge::Left => (2, mid),
                    Edge::Right => (w - 3, mid),
                    Edge::Top => (mid, 2),
                    Edge::Bottom => (mid, h - 3),
                }
            })
            .collect()
    }
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
