use std::fmt;

use derivative::Derivative;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct UnitHashTable {
    pub player_ptrs: [u64; 128],
    pub npc_ptrs: [u64; 128],
    pub object_ptrs: [u64; 128],
    pub missile_ptrs: [u64; 128],
    pub item_ptrs: [u64; 128],
    pub tile_ptrs: [u64; 128],
    pub server_player_ptrs: [u64; 128],
    pub server_npc_ptrs: [u64; 128],
    pub server_object_ptrs: [u64; 128],
    pub server_missile_ptrs: [u64; 128],
}

impl Default for UnitHashTable {
    fn default() -> UnitHashTable {
        UnitHashTable {
            player_ptrs: [0; 128],
            npc_ptrs: [0; 128],
            object_ptrs: [0; 128],
            missile_ptrs: [0; 128],
            item_ptrs: [0; 128],
            tile_ptrs: [0; 128],
            server_player_ptrs: [0; 128],
            server_npc_ptrs: [0; 128],
            server_object_ptrs: [0; 128],
            server_missile_ptrs: [0; 128],
        }
    }
}

#[repr(C)]
#[derive(Derivative, Debug, Copy, Clone)]
#[derivative(Default)]
pub struct Unit {
    pub unit_type: u32,
    pub txt_file_no: u32,
    pub unit_id: u32,
    pub mode: u32,
    pub p_unit_data: u64,
    _dummy: u64,
    pub p_act: u64,
    _dummy2: u64,
    _dummy3: u64,
    pub p_path: u64,
    #[derivative(Default(value = "[0; 72]"))]
    _dummy4: [u8; 72],  //0x5e
    pub p_stats_list_ex: u64,
    pub p_inventory: u64,
    #[derivative(Default(value = "[0; 16]"))]
    _dummy5: [u8; 16],        //0x0098
    _dummy6: usize,               //0x00A8
    #[derivative(Default(value = "[0; 40]"))]
    _dummy7: [u8; 40],        //0x00B0
    pub unk_sort_stashes_by: u32,  //0x00D8
    #[derivative(Default(value = "[0; 36]"))]
    _dummy8: [u8; 36],
    pub p_skills: u64,
    #[derivative(Default(value = "[0; 78]"))]
    _dummy9: [u8; 78],
    pub p_next: u64,
    pub p_room_next: u64,
    #[derivative(Default(value = "[0; 20]"))]
    _dummy10: [u8; 20],
    pub player_class: u32, //0x174
    #[derivative(Default(value = "[0; 46]"))]
    _dummy11: [u8; 46],
    pub is_corpse: u8, //0x1A6
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct Path {
    pub offset_x: u16,
    pub dynamic_x: u16,
    pub offset_y: u16,
    pub dynamic_y: u16,
    pub static_x: u16,
    _dummy: u16,
    pub static_y: u16,
    _dummy2: u16,
    _dummy3: u64,
    _dummy4: u64,
    pub p_room: u64,
}

#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct StaticPath {
    pub room: u64,
    target_x: u32,
    target_y: u32,
    pub x: u32,
    pub y: u32,
}

#[repr(C)]
#[derive(Debug, Derivative, Copy, Clone)]
#[derivative(Default)]
pub struct StatsList {
    #[derivative(Default(value = "[0; 48]"))]
    _dummy: [u8; 48],
    pub stat_ptr: u64,  // 0x30
    pub stat_count: u32,
    #[derivative(Default(value = "[0; 84]"))]
    _dummy2: [u8; 84],
    pub stat_unit_ptr: u64, // 0x90
    #[derivative(Default(value = "[0; 16]"))]
    _dummy3: [u8; 16],
    pub stat_ex_ptr: u64,  // 0xA8
    pub stat_ex_count: u32, // 0xB0
    #[derivative(Default(value = "[0; 2620]"))]
    _dummy4: [u8; 2620],
    pub state_flags: [u32; 6],  //0xAF0
}

impl fmt::Display for StatsList {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "stat_ptr {} stat_count {} stat_ex_ptr {} stat_ex_count {} state_flaags {:?}", self.stat_ptr, self.stat_count, self.stat_ex_ptr, self.stat_ex_count, self.state_flags)
    }
}

#[derive(Clone, Debug, Default)]
pub struct StatValueStruct {
    pub layer: u16,
    pub stat: u16,
    pub value: i16,
    pub value2: i16,
}

#[repr(C)]
#[derive(Derivative, Debug, Copy, Clone)]
#[derivative(Default)]
pub struct MonsterData {
    pub p_mon_stats: u64,
    _dummy: [u8; 18],
    pub monster_flags: u8,
    pub last_animation_mode: u8,
    pub duriel_flag: u32,
    pub mon_u_mod: [u8; 10],
    pub boss_line_id: u16,
    _dummy3: [u8; 6],
    pub merc_name: u16,
}

#[repr(C)]
#[derive(Derivative, Debug, Copy, Clone)]
#[derivative(Default)]
pub struct ObjectData {
    pub p_mon_stats: u64,
    pub interact_type: u8,  //interact type is also the area number of the portal
    pub shrine_flag: u8,
    _dummy: [u8; 2],
    pub shrine_txt: u64,
    #[derivative(Default(value = "[0; 32]"))]
    _dummy2: [u8; 32],
    pub owner: [u8; 16],
}

#[repr(C)]
#[derive(Derivative, Debug, Copy, Clone)]
#[derivative(Default)]
pub struct MissileData {
    #[derivative(Default(value = "[0; 14]"))]
    _dummy: [u8; 14],
    pub skill_level: u8,      //0xe
    _dummy2: u8,
    pub base_skill_level: u8, //0x10
    #[derivative(Default(value = "[0; 19]"))]
    _dummy3: [u8; 19],
    pub dw_owner_id: u32,     //0x24
}


#[repr(C)]
#[derive(Derivative, Debug, Copy, Clone)]
#[derivative(Default)]

pub struct ItemData {
    pub quality: u32,
    low_seed: u32,
    high_seed: u32,
    pub dw_owner_id: u32,
    init_seed: u32,
    pub command_flags: u32,
    pub flags: u32,
    _dummy4: [u8; 24],
    pub file_text: u32,
    pub item_level: u32,
    _dummy5: [u8; 4],
    pub item_format: u16,
    pub rare_prefix: u16,
    pub rare_suffix: u16,
    pub auto_affix: u16,
    pub magic_prefix: [u16; 3],
    pub magic_suffix: [u16; 3],
    pub body_loc: u8,
    pub inv_page: u8,
}

#[repr(C)]
#[derive(Derivative, Debug, Copy, Clone)]
#[derivative(Default)]
pub struct Act {
    #[derivative(Default(value = "[0; 112]"))]
    _dummy: [u8; 112],
    pub p_act_misc: u64,
}

#[repr(C)]
#[derive(Derivative, Debug, Copy, Clone)]
#[derivative(Default)]
pub struct ActMisc {
    #[derivative(Default(value = "[0; 2096]"))]
    _dummy: [u8; 2096],
    pub game_difficulty: u32,
    #[derivative(Default(value = "[0; 12]"))]
    _dummy2: [u8; 12],
    pub dw_init_seed_hash: u64,  // 2112
    #[derivative(Default(value = "[0; 24]"))]
    _dummy3: [u8; 24],
    pub dw_end_seed_hash: u32,
    #[derivative(Default(value = "[0; 24]"))]
    _dummy4: [u8; 24],

    
}

#[repr(C)]
#[derive(Derivative, Debug, Copy, Clone)]
#[derivative(Default)]
pub struct Room {
    #[derivative(Default(value = "[0; 24]"))]
    _dummy: [u8; 24],
    pub p_room_2: u64,
}

#[repr(C)]
#[derive(Derivative, Debug, Copy, Clone)]
#[derivative(Default)]
pub struct Room2 {
    #[derivative(Default(value = "[0; 144]"))]
    _dummy: [u8; 144],
    pub p_level: u64,
}

#[repr(C)]
#[derive(Derivative, Debug, Copy, Clone)]
#[derivative(Default)]
pub struct Level {
    #[derivative(Default(value = "[0; 504]"))]
    _dummy: [u8; 504],
    pub level_id: u32,
}

#[repr(C)]
#[derive(Derivative, Debug, Copy, Clone)]
#[derivative(Default)]
pub struct Roster {
    #[derivative(Default(value = "[0; 48]"))]
    pub name: [u8; 48],
    #[derivative(Default(value = "[0; 24]"))]
    pad_0010: [u8; 24],
    pub dw_unit_id: u32,
    pad_004c: [u8; 8],
    pub player_class: u32,
    pub player_level: u16,
    pub party_id: u16,
    pub area: u32,
    pub pos_x: u32,
    pub pos_y: u32,
    pub party_flags: u32,
    pad_003c: [u8; 4],
    pub hostile_info: u64,
    #[derivative(Default(value = "[0; 208]"))]
    pad_0048: [u8; 208],
    pub next_roster: u64,
}


#[repr(C)]
#[derive(Derivative, Debug, Copy, Clone)]
#[derivative(Default)]
pub struct HostileInfo {
    pub dw_unit_id: u32,
    pub hostile_flag: u32,
    pub next_hostile_info: u64,
}

#[repr(C)]
#[derive(Derivative, Debug, Copy, Clone)]
#[derivative(Default)]
pub struct MenuStates {
    pub in_game: bool,                  //0x0000  totally didn't copy and paste this from anywhere
    pub inventory_visible: bool,        //0x0001
    pub stats_visible: bool,            //0x0002
    pub skill_popover_visible: bool,    //0x0003
    pub skill_tree_visible: bool,       //0x0004
    pub chat_visible: bool,             //0x0005
    _n000012bc: bool,                   //0x0006
    _n000012cf: bool,                   //0x0007
    pub npc_dialog_visible: bool,       //0x0008
    pub pause_menu_visible: bool,       //0x0009
    pub automap_visible: bool,          //0x000A
    pub vendor_inventory_visible: bool, //0x000B
    pub items_on_ground_visible: bool,  //0x000C
    _n000012d5: bool,                   //0x000D
    pub quest_menu_visible: bool,       //0x000E
    n000012d7: bool,                    //0x000F
    n00001256: bool,                    //0x0010
    n000012e1: bool,                    //0x0011
    pub party_portaits: bool,           //0x0012
    pub waypoint_menu_visible: bool,    //0x0013
    _unk2visible: bool,                 //0x0014 1 when nothing is open - changes to 0 when pause menu is opened
    pub party_visible: bool,            //0x0015
    n00001310: bool,                    //0x0016
    n000012e3: bool,                    //0x0017
    pub stash_visible: bool,            //0x0018
    pub cube_visible: bool,             //0x0019
    pub belt_visible: bool,             //0x001A
    n00001353: bool,                    //0x001B
    n00001314: bool,                    //0x001C
    pub avatars_visible: bool,          //0x001D
    pub merc_inventory_visible: bool,   //0x001E
}

#[allow(dead_code)]
#[repr(C)]
#[derive(Derivative, Debug, Copy, Clone)]
#[derivative(Default)]
pub struct CameraWrapper {
    #[derivative(Default(value = "[0; 40]"))]
    pad_0000: [u8; 40],
    pub camera: u64,
}

#[allow(dead_code)]
#[repr(C)]
#[derive(Derivative, Debug, Copy, Clone)]
#[derivative(Default)]
pub struct Camera {
    #[derivative(Default(value = "[0; 44]"))]
    pad_0000: [u8; 44],
    pub current_zoom: f32,
    pub zoom_amount: f32,
}

#[repr(C)]
#[derive(Derivative, Debug, Copy, Clone)]
#[derivative(Default)]
pub struct UIWidget {
    dummy: usize,
    pub p_name: u64,
    pub name_length: u64,
    name_max_length: u32,
    text_flags: u32,
    pub inline_name: [u8; 16],
    pub parent: u64,
    pad_0038: [u8; 16],
    pub rel_position_x: f32,
    pub rel_position_y: f32,
    pub enabled: bool,
    pub visible: bool,
    pad_0052: [u8; 6],
    pub children: u64,  // 96c
    pub num_children: u64,
    pub allocated: u64,
}


#[repr(C)]
#[derive(Derivative, Debug, Copy, Clone)]
#[derivative(Default)]
pub struct UIPanelManager {
    vmt: usize,
    pub name: u64,
    pub name_length: u64,
    name_max_length: u32,
    text_flags: u32,
    pub inline_name: [u8; 16],
    pub parent: u64,
    pad_0038: [u8; 16],
    pub rel_position_x: f32,
    pub rel_position_y: f32,
    pub enabled: bool,
    pub visible: bool,
    pad_0052: [u8; 6],
    pub children: u64,
    pub num_children: u64,
    pub allocated: u64,
}



#[repr(C)]
#[derive(Derivative, Debug, Copy, Clone)]
#[derivative(Default)]
pub struct SkillList {
    pub p_first_skill: u64,
    pub p_left_skill: u64,
    pub p_right_skill: u64,
    pub p_used_skill: u64,
}


#[repr(C)]
#[derive(Derivative, Debug, Copy, Clone)]
#[derivative(Default)]
pub struct SkillStrc {
    pub p_skill_txt: u64,
    pub p_next_skill: u64,
    #[derivative(Default(value = "[0; 40]"))]
    pub _dummy: [u8; 40],
    pub hard_points: u32,
    pub _dummy2: u32,
    pub quantity: u32,
    pub _dummy3: u32,
    pub charges: u32,
}

#[repr(C)]
#[derive(Derivative, Debug, Copy, Clone)]
#[derivative(Default)]
pub struct SkillTxt {
    pub id: i16,
}
