//! Live map for levels that libd2 cannot generate (mod-added custom levels, IDs above 136,
//! e.g. Reimagined's Forsaken Labyrinth, RMAP maps and boss arenas).
//!
//! Those levels are built by the mod's own plugin (d2rl-maps.dll) with per-map seeds, so they
//! cannot be generated offline. Instead we read what the game itself has loaded:
//!
//!   player -> Path -> Room1 (active room) -> CollMap (subtile collision for that room)
//!                              \-> rooms-near list -> more Room1s ...
//!   Room1 -> Room2 (+0x18) -> Level (+0x90) -> level id (+0x1F8)
//!
//! Every room we have ever seen is remembered for the rest of the game, so the map fills in
//! as you explore - like the game's own automap, but shown as PrimeMH's outline.
//!
//! The Room1 -> CollMap and rooms-near field offsets are not publicly documented for D2R, so
//! they are found at runtime: we look for a pointer in the player's room that leads to a
//! collision block whose rectangle contains the player and whose cell under the player is
//! walkable. The result is logged as "Live map layout found ..." in the log file.

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use crate::mapgeneration::jsondata::{LevelData, Offset, SeedData, Size};
use crate::mapgeneration::libd2gen::{encode_row, LAST_VANILLA_LEVEL};
use crate::memory::process::D2RInstance;
use crate::memory::structs::Path;
use crate::types::player::PlayerUnit;

const ROOM1_TO_ROOM2: u64 = 0x18;
const ROOM2_TO_LEVEL: u64 = 0x90;
const LEVEL_TO_ID: u64 = 0x1F8;

const MAX_ROOMS_PER_SCAN: usize = 768;
const SCAN_INTERVAL: Duration = Duration::from_millis(400);
const BUILD_INTERVAL: Duration = Duration::from_millis(1000);

#[derive(Clone, Copy, Debug)]
struct Layout {
    /// Room1 field holding the CollMap pointer
    coll_ptr: u64,
    /// offset of posX,posY,sizeX,sizeY (u32, subtiles) inside CollMap
    coll_pos: u64,
    /// offset of the u16 cell buffer pointer inside CollMap
    map_ptr: u64,
    /// Room1 field holding the Room1** rooms-near array, and its u32 count
    near_ptr: u64,
    near_count: u64,
}

#[derive(Clone, PartialEq)]
struct LiveRoom {
    level: u32,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    blocked: Vec<bool>,
}

pub struct LiveMap {
    layout: Option<Layout>,
    rooms: HashMap<(u32, u32, u32), LiveRoom>,
    dirty: HashSet<u32>,
    seed: u32,
    last_scan: Instant,
    last_build: Instant,
    last_fail_log: Option<Instant>,
    last_unit_log: Option<Instant>,
}

impl Default for LiveMap {
    fn default() -> Self {
        LiveMap {
            layout: None,
            rooms: HashMap::new(),
            dirty: HashSet::new(),
            seed: 0,
            last_scan: Instant::now() - SCAN_INTERVAL,
            last_build: Instant::now() - BUILD_INTERVAL,
            last_fail_log: None,
            last_unit_log: None,
        }
    }
}

/// True if this level has to come from the live map rather than from libd2.
pub fn needs_live_map(level_id: u32, seed_data: &SeedData) -> bool {
    level_id > LAST_VANILLA_LEVEL || !seed_data.levels.iter().any(|l| l.id == level_id)
}

fn looks_like_ptr(p: u64) -> bool {
    p > 0x10000 && p < 0x7FFF_FFFF_FFFF && p % 8 == 0
}

fn u32_at(b: &[u8], off: usize) -> u32 {
    u32::from_le_bytes(b[off..off + 4].try_into().unwrap())
}

fn u64_at(b: &[u8], off: usize) -> u64 {
    u64::from_le_bytes(b[off..off + 8].try_into().unwrap())
}

fn read_cells(d2r: &D2RInstance, ptr: u64, count: usize) -> Option<Vec<u16>> {
    let bytes = d2r.read_bytes(ptr, count * 2)?;
    Some(bytes.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect())
}

fn sane_size(w: u32, h: u32) -> bool {
    (1..=1000).contains(&w) && (1..=1000).contains(&h)
}

impl LiveMap {
    /// Call once per frame while in game. Cheap when the current level is a vanilla one.
    /// Returns true if `seed_data` was changed (new or updated live levels were inserted).
    pub fn update(&mut self, d2r: &D2RInstance, player: &PlayerUnit, seed: u32, current_level: u32, seed_data: &mut SeedData) -> bool {
        if seed != self.seed {
            self.rooms.clear();
            self.dirty.clear();
            self.seed = seed;
        }
        let live_levels_present = seed_data.levels.iter().any(|l| l.level_type == "live");
        if !needs_live_map(current_level, seed_data) && !live_levels_present {
            return false;
        }
        if self.last_scan.elapsed() >= SCAN_INTERVAL {
            self.last_scan = Instant::now();
            self.scan(d2r, player, seed_data);
        }
        if self.dirty.is_empty() || self.last_build.elapsed() < BUILD_INTERVAL {
            return false;
        }
        self.last_build = Instant::now();

        let dirty: Vec<u32> = self.dirty.drain().collect();
        let mut changed = false;
        for level_id in dirty {
            if let Some(level) = self.build_level(level_id) {
                seed_data.levels.retain(|l| l.id != level_id);
                seed_data.levels.push(level);
                changed = true;
            }
        }
        changed
    }

    /// While on a custom level, log what kinds of monsters PrimeMH can see (every 15s),
    /// so missing monster dots can be diagnosed from primemh.log.
    pub fn log_units(&mut self, current_level: u32, npcs: &[crate::types::npc::NPCUnit]) {
        if current_level <= LAST_VANILLA_LEVEL {
            return;
        }
        if self.last_unit_log.map_or(false, |t| t.elapsed() < Duration::from_secs(15)) {
            return;
        }
        self.last_unit_log = Some(Instant::now());
        let mut counts: HashMap<String, usize> = HashMap::new();
        for n in npcs {
            *counts.entry(format!("{:?}/{:?}", n.npc_type, n.monster_flag)).or_default() += 1;
        }
        let unknown = npcs.iter().filter(|n| matches!(n.txt_file_no, crate::types::npc::NPC::Unknown)).count();
        log::info!("Live map level {}: {} units, {} with mod-added ids, by type {:?}", current_level, npcs.len(), unknown, counts);
    }

    fn player_room(d2r: &D2RInstance, player: &PlayerUnit) -> u64 {
        let path: Path = d2r.read_mem::<Path>(player.raw.p_path);
        path.p_room
    }

    fn scan(&mut self, d2r: &D2RInstance, player: &PlayerUnit, seed_data: &SeedData) {
        let start_room = Self::player_room(d2r, player);
        if !looks_like_ptr(start_room) {
            return;
        }
        if self.layout.is_none() {
            self.layout = discover_layout(d2r, start_room, player.pos_x as u32, player.pos_y as u32);
            match self.layout {
                Some(l) => log::info!("Live map layout found: {:x?}", l),
                None => {
                    if self.last_fail_log.map_or(true, |t| t.elapsed() > Duration::from_secs(10)) {
                        log::warn!("Live map: could not find the room collision layout yet (custom level maps unavailable)");
                        self.last_fail_log = Some(Instant::now());
                    }
                    return;
                }
            }
        }
        let layout = self.layout.unwrap();

        // breadth-first walk over every loaded room reachable from the player's room
        let mut seen: HashSet<u64> = HashSet::new();
        let mut queue: Vec<u64> = vec![start_room];
        seen.insert(start_room);
        let mut i = 0;
        let mut ok_rooms = 0usize;
        while i < queue.len() && queue.len() <= MAX_ROOMS_PER_SCAN {
            let room = queue[i];
            i += 1;
            let Some(head) = d2r.read_bytes(room, 0x100) else { continue };
            if let Some(live) = read_room(d2r, room, &head, &layout) {
                ok_rooms += 1;
                // only keep levels libd2 does not already provide
                if live.level > 0 && (live.level > LAST_VANILLA_LEVEL || !seed_data.levels.iter().any(|l| l.id == live.level && l.level_type != "live")) {
                    let key = (live.level, live.x, live.y);
                    if self.rooms.get(&key) != Some(&live) {
                        self.dirty.insert(live.level);
                        self.rooms.insert(key, live);
                    }
                }
            }
            let count = u32_at(&head, layout.near_count as usize) as usize;
            let arr = u64_at(&head, layout.near_ptr as usize);
            if count == 0 || count > 64 || !looks_like_ptr(arr) {
                continue;
            }
            if let Some(near) = d2r.read_bytes(arr, count * 8) {
                for n in 0..count {
                    let r = u64_at(&near, n * 8);
                    if looks_like_ptr(r) && seen.insert(r) {
                        queue.push(r);
                    }
                }
            }
        }
        if ok_rooms == 0 {
            // layout went stale (game update?) - rediscover next time
            self.layout = None;
        }
    }

    fn build_level(&self, level_id: u32) -> Option<LevelData> {
        let rooms: Vec<&LiveRoom> = self.rooms.values().filter(|r| r.level == level_id).collect();
        if rooms.is_empty() {
            return None;
        }
        let min_x = rooms.iter().map(|r| r.x).min()?;
        let min_y = rooms.iter().map(|r| r.y).min()?;
        let max_x = rooms.iter().map(|r| r.x + r.w).max()?;
        let max_y = rooms.iter().map(|r| r.y + r.h).max()?;
        let width = (max_x - min_x) as usize;
        let height = (max_y - min_y) as usize;
        if width == 0 || height == 0 || width > 4000 || height > 4000 {
            return None;
        }
        let mut blocked = vec![true; width * height];
        for r in rooms {
            let ox = (r.x - min_x) as usize;
            let oy = (r.y - min_y) as usize;
            for yy in 0..r.h as usize {
                for xx in 0..r.w as usize {
                    if !r.blocked[yy * r.w as usize + xx] {
                        blocked[(oy + yy) * width + ox + xx] = false;
                    }
                }
            }
        }
        let map: Vec<Vec<u64>> = (0..height)
            .map(|y| encode_row(blocked[y * width..(y + 1) * width].iter().copied()))
            .collect();
        Some(LevelData {
            // "live" marks levels built from game memory; the drawing code treats it like "map"
            level_type: "live".to_owned(),
            id: level_id,
            name: Default::default(),
            offset: Offset { x: min_x, y: min_y },
            size: Size { width: width as u32, height: height as u32 },
            objects: vec![],
            map,
            level_image: Default::default(),
        })
    }
}

fn level_of_room(d2r: &D2RInstance, head: &[u8]) -> u32 {
    let room2 = u64_at(head, ROOM1_TO_ROOM2 as usize);
    if !looks_like_ptr(room2) {
        return 0;
    }
    let level: u64 = d2r.read_mem::<u64>(room2 + ROOM2_TO_LEVEL);
    if !looks_like_ptr(level) {
        return 0;
    }
    d2r.read_mem::<u32>(level + LEVEL_TO_ID)
}

fn read_room(d2r: &D2RInstance, _room: u64, head: &[u8], l: &Layout) -> Option<LiveRoom> {
    let coll = u64_at(head, l.coll_ptr as usize);
    if !looks_like_ptr(coll) {
        return None;
    }
    let cb = d2r.read_bytes(coll, 0x60)?;
    let p = l.coll_pos as usize;
    let (x, y, w, h) = (u32_at(&cb, p), u32_at(&cb, p + 4), u32_at(&cb, p + 8), u32_at(&cb, p + 12));
    if !sane_size(w, h) {
        return None;
    }
    let mp = u64_at(&cb, l.map_ptr as usize);
    if !looks_like_ptr(mp & !7) {
        return None;
    }
    let cells = read_cells(d2r, mp, (w * h) as usize)?;
    Some(LiveRoom {
        level: level_of_room(d2r, head),
        x,
        y,
        w,
        h,
        blocked: cells.iter().map(|c| c & 1 != 0).collect(),
    })
}

/// Find where the collision data and the rooms-near list live in a D2R Room1.
fn discover_layout(d2r: &D2RInstance, room: u64, px: u32, py: u32) -> Option<Layout> {
    let head = d2r.read_bytes(room, 0x100)?;

    // 1. collision: a pointer in Room1 -> block with a rectangle containing the player
    let mut best: Option<(u64, u64, u64, bool)> = None;
    'outer: for o in (0..0x100).step_by(8) {
        if o as u64 == ROOM1_TO_ROOM2 {
            continue;
        }
        let p = u64_at(&head, o);
        if !looks_like_ptr(p) {
            continue;
        }
        let Some(cb) = d2r.read_bytes(p, 0x60) else { continue };
        for k in (0..=0x10).step_by(4) {
            let (gx, gy, sx, sy) = (u32_at(&cb, k), u32_at(&cb, k + 4), u32_at(&cb, k + 8), u32_at(&cb, k + 12));
            if !sane_size(sx, sy) || px < gx || py < gy || px >= gx + sx || py >= gy + sy {
                continue;
            }
            let first_m = ((k + 16) + 7) & !7;
            for m in (first_m..0x58).step_by(8) {
                let mp = u64_at(&cb, m);
                if !looks_like_ptr(mp & !7) {
                    continue;
                }
                let Some(cells) = read_cells(d2r, mp, (sx * sy) as usize) else { continue };
                let under = cells[((py - gy) * sx + (px - gx)) as usize];
                if under & 1 != 0 {
                    continue; // the player cannot be standing in a wall
                }
                let end = u64_at(&cb, m + 8);
                let strong = end > mp && end - mp == (sx * sy * 2) as u64;
                if best.map_or(true, |b| !b.3 && strong) {
                    best = Some((o as u64, k as u64, m as u64, strong));
                }
                if strong {
                    break 'outer;
                }
            }
        }
    }
    let (coll_ptr, coll_pos, map_ptr, _) = best?;
    let partial = Layout { coll_ptr, coll_pos, map_ptr, near_ptr: 0, near_count: 0x40 };

    // 2. rooms-near: an array of Room1 pointers + a small count. Known D2R layout first.
    let mut candidates: Vec<(u64, u64)> = vec![(0x00, 0x40)];
    for np in (0..0x100u64).step_by(8) {
        for nc in (0..0x100u64).step_by(4) {
            if (np, nc) != (0x00, 0x40) {
                candidates.push((np, nc));
            }
        }
    }
    for (np, nc) in candidates {
        let arr = u64_at(&head, np as usize);
        let count = u32_at(&head, nc as usize) as usize;
        if !looks_like_ptr(arr) || count == 0 || count > 64 {
            continue;
        }
        let Some(near) = d2r.read_bytes(arr, count * 8) else { continue };
        let mut good = 0;
        for n in 0..count {
            let r = u64_at(&near, n * 8);
            if !looks_like_ptr(r) {
                continue;
            }
            if let Some(h) = d2r.read_bytes(r, 0x100) {
                if read_room(d2r, r, &h, &partial).is_some() {
                    good += 1;
                }
            }
        }
        if good * 2 >= count && good >= 1 {
            return Some(Layout { near_ptr: np, near_count: nc, ..partial });
        }
    }
    // collision works but no neighbour list: still map the room the player stands in
    Some(partial)
}
