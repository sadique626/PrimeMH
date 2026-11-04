use super::{process::D2RInstance, structs::{MenuStates, UnitHashTable}};
use crate::types::{
     get_missiles, get_units, item::ItemUnit, keybindings::KeyBindings, last_hovered::LastHovered, missile::MissileUnit, npc::NPCUnit, object::GameObjectUnit, player::{self, PlayerUnit}, roster::{self, RosterItem}, seedvalues::SeedValues
};

#[derive(Debug, Clone)]
pub struct GroundItem {
    pub unit_id: u32,
    pub txt_file_no: u32,
    /// 1 low, 2 normal, 3 superior, 4 magic, 5 set, 6 rare, 7 unique, 8 crafted
    pub quality: u32,
    /// which unique / set item it is (uniqueitems.txt / setitems.txt *ID), set before identifying
    pub file_index: u32,
    pub x: u32,
    pub y: u32,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct GameData {
    pub seed_values: SeedValues,
    pub player: PlayerUnit,
    pub players: Vec<PlayerUnit>,
    pub npcs: Vec<NPCUnit>,
    pub roster_items: Vec<RosterItem>,
    pub objects: Vec<GameObjectUnit>,
    pub items: Vec<ItemUnit>,
    pub missiles: Vec<MissileUnit>,
    pub menus: MenuStates,
    pub last_hovered: LastHovered,
    pub keybindings: KeyBindings,
    /// world positions of warp tiles (stairs, doors, cave mouths) in the rooms the game has loaded
    pub warp_tiles: Vec<(u32, u32)>,
    /// the same warp tiles with the room (Room1) each sits in: (x, y, room)
    pub warp_tile_rooms: Vec<(u32, u32, u64)>,
    /// the player's total experience (read as a full 32-bit value)
    pub player_experience: Option<u32>,
    /// unique and set relevant info of items lying on the ground
    pub ground_items: Vec<GroundItem>,
}

impl GameData {
    pub fn read_game_memory(d2rprocess: &D2RInstance) -> Option<GameData> {
        let unit_ptrs: UnitHashTable = d2rprocess.read_mem_offset::<UnitHashTable>(d2rprocess.offsets.unit_table);
        let players: Vec<PlayerUnit> = get_units(d2rprocess, unit_ptrs.player_ptrs);
        if players.is_empty() {
            // not in a current game
            // println!("No players");
            return None;
        }
        let roster_items: Vec<RosterItem> = roster::get_roster(d2rprocess);
        let first_roster = roster_items.get(0);
        let player_roster = match first_roster {
            Some(player_roster) => player_roster,
            None => return None,
        };
        let player: &PlayerUnit = match player::get_current_player(&players, player_roster.unit_id) {
            Some(p) => p,
            None => {
                // println!("{} No player found", players.len());
                return None;
            }
        };

        let seed_values = SeedValues::new(d2rprocess, player);
        if seed_values.dw_init_seed_hash == 0 {
            return None;
            
        }
        let npcs: Vec<NPCUnit> = get_units(d2rprocess, unit_ptrs.npc_ptrs);
        let objects: Vec<GameObjectUnit> = get_units(d2rprocess, unit_ptrs.object_ptrs);
        // item log / item filter removed: items are no longer read from memory
        let items: Vec<ItemUnit> = Vec::new();
        let missiles: Vec<MissileUnit> = get_missiles(d2rprocess, unit_ptrs.missile_ptrs, unit_ptrs.server_missile_ptrs, player);

        // let panels = get_panels(d2rprocess);

        let menus: MenuStates = MenuStates::get_menu_states(d2rprocess);
        let last_hovered: LastHovered = LastHovered::get_hovered(d2rprocess);

        let keybindings: KeyBindings = KeyBindings::new(d2rprocess);
        let player_experience = read_experience(d2rprocess, &player.raw);
        let warp_tile_rooms: Vec<(u32, u32, u64)> = crate::types::get_raw_units(d2rprocess, unit_ptrs.tile_ptrs)
            .into_iter()
            .filter(|u| u.p_path != 0)
            .map(|u| {
                let sp: crate::memory::structs::StaticPath = d2rprocess.read_mem(u.p_path);
                (sp.x, sp.y, sp.room)
            })
            .filter(|p| p.0 > 0 && p.1 > 0)
            .collect();
        let warp_tiles: Vec<(u32, u32)> = warp_tile_rooms.iter().map(|t| (t.0, t.1)).collect();
        

        Some(GameData {
            seed_values,
            player: player.clone(),
            players,
            npcs,
            roster_items,
            objects,
            items,
            missiles,
            menus,
            last_hovered,
            keybindings,
            warp_tiles,
            warp_tile_rooms,
            player_experience,
            ground_items: read_ground_items(d2rprocess, unit_ptrs.item_ptrs),
        })
    }

    #[allow(unused)]
    pub fn is_in_town(&self) -> bool {
        match self.seed_values.level {
            1 | 40 | 75 | 103 | 109 => true,
            _ => false
        }
    }
}


pub fn get_last_game_name(d2rprocess: &D2RInstance) -> String {
    let last_game_arr = d2rprocess.read_mem_offset::<[u8; 15]>(d2rprocess.offsets.last_game_name);
    let mut game_name = d2rprocess.parse_arr_to_string(&last_game_arr);

    // there's a problem with non-english characters sometimes
    if !game_name.chars().all(|x| x.is_ascii()) {
        game_name = "".to_string();
    }
    game_name
}

/// Experience is stat 13. Each stat entry is { u16 layer, u16 stat, u32 value }; read the value
/// as a whole u32, since experience is far beyond what a 16-bit stat value can hold.
fn read_experience(d2rprocess: &D2RInstance, unit: &super::structs::Unit) -> Option<u32> {
    use std::sync::atomic::{AtomicU32, Ordering};
    static DIAG: AtomicU32 = AtomicU32::new(0);
    if unit.p_stats_list_ex == 0 {
        if DIAG.fetch_add(1, Ordering::Relaxed) == 0 {
            log::info!("XP diag: player has no stats list pointer");
        }
        return None;
    }
    // Collect every (pointer, count) pair that could be a stat array: anything in the first
    // 0x100 bytes of the stats list, plus the chain of stat lists hanging off it (+0x90).
    let mut arrays: Vec<(String, u64, u32)> = vec![];
    let scan = |base: u64, tag: &str, arrays: &mut Vec<(String, u64, u32)>| {
        if let Some(b) = d2rprocess.read_bytes(base, 0x100) {
            for off in (0..0xF0).step_by(8) {
                let ptr = u64::from_le_bytes(b[off..off + 8].try_into().unwrap());
                let cnt = u32::from_le_bytes(b[off + 8..off + 12].try_into().unwrap());
                if ptr > 0x10000 && ptr < 0x7FFF_FFFF_FFFF && cnt > 0 && cnt <= 512 {
                    arrays.push((format!("{}+0x{:X}", tag, off), ptr, cnt));
                }
            }
        }
    };
    scan(unit.p_stats_list_ex, "list", &mut arrays);
    let mut p = d2rprocess.read_mem::<u64>(unit.p_stats_list_ex + 0x90);
    for i in 0..8 {
        if p < 0x10000 || p == unit.p_stats_list_ex {
            break;
        }
        scan(p, &format!("chain{}", i), &mut arrays);
        p = d2rprocess.read_mem::<u64>(p + 0x48);
    }
    let n = DIAG.fetch_add(1, Ordering::Relaxed);
    let diag = n < 3 || n % 3600 == 0;
    let mut best: Option<u32> = None;
    let mut any_xp: Option<u32> = None;
    let mut report: Vec<String> = vec![];
    for (tag, ptr, count) in arrays {
        // D2RLoader stores 12-byte stat entries {u16 stat, u16 layer, u32 value, u32 extra};
        // the original game uses 8-byte {u16 layer, u16 stat, u32 value}. Try both.
        let Some(bytes) = d2rprocess.read_bytes(ptr, count as usize * 12) else { continue };
        let mut level = None;
        let mut xp = None;
        let mut layout = "";
        for (stride, stat_at, name) in [(12usize, 0usize, "12-byte"), (8, 2, "8-byte")] {
            let mut l = None;
            let mut x = None;
            for e in bytes.chunks_exact(stride).take(count as usize) {
                let stat = u16::from_le_bytes([e[stat_at], e[stat_at + 1]]);
                let other = if stat_at == 0 { u16::from_le_bytes([e[2], e[3]]) } else { u16::from_le_bytes([e[0], e[1]]) };
                if other != 0 {
                    continue; // layer must be 0 for level / experience
                }
                let v = u32::from_le_bytes([e[4], e[5], e[6], e[7]]);
                match stat {
                    12 => l = Some(v),
                    13 => x = Some(x.map_or(v, |o: u32| o.max(v))),
                    _ => {}
                }
            }
            if l.map_or(false, |v| (1..=200).contains(&v)) && x.is_some() {
                level = l;
                xp = x;
                layout = name;
                break;
            }
            if xp.is_none() && x.is_some() {
                level = l;
                xp = x;
                layout = name;
            }
        }
        if diag && (level.is_some() || xp.is_some()) {
            report.push(format!("{} n={} {} level={:?} xp={:?}", tag, count, layout, level, xp));
        }
        if let (Some(x), true) = (xp, tag == "list+0x30" || tag == "list+0xA8") {
            any_xp = Some(any_xp.map_or(x, |b| b.max(x)));
        }
        // a real player stat array has the level next to the experience
        if let (Some(l), Some(x)) = (level, xp) {
            if (1..=200).contains(&l) {
                best = Some(best.map_or(x, |b| b.max(x)));
            }
        }
    }
    if diag {
        log::info!("XP diag: {}", if report.is_empty() { "no stat array with level/xp found".to_string() } else { report.join(" | ") });
    }
    best.or(any_xp)
}

/// Items on the ground (mode 3) or dropping (mode 5). ItemData: quality at 0x00, file index at 0x34.
fn read_ground_items(d2rprocess: &D2RInstance, item_ptrs: [u64; 128]) -> Vec<GroundItem> {
    crate::types::get_raw_units(d2rprocess, item_ptrs)
        .into_iter()
        .filter(|u| (u.mode == 3 || u.mode == 5) && u.p_unit_data != 0)
        .filter_map(|u| {
            let data = d2rprocess.read_bytes(u.p_unit_data, 0x38)?;
            let quality = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
            let file_index = u32::from_le_bytes([data[0x34], data[0x35], data[0x36], data[0x37]]);
            let path: crate::memory::structs::StaticPath = d2rprocess.read_mem(u.p_path);
            Some(GroundItem { unit_id: u.unit_id, txt_file_no: u.txt_file_no, quality, file_index, x: path.x, y: path.y })
        })
        .collect()
}
