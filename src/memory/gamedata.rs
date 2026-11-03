use super::{process::D2RInstance, structs::{MenuStates, UnitHashTable}};
use crate::types::{
     get_missiles, get_units, item::ItemUnit, keybindings::KeyBindings, last_hovered::LastHovered, missile::MissileUnit, npc::NPCUnit, object::GameObjectUnit, player::{self, PlayerUnit}, roster::{self, RosterItem}, seedvalues::SeedValues
};

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
