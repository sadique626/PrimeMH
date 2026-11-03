use std::mem::transmute;

use crate::{
    mapgeneration::jsondata::LevelName,
    memory::{process::D2RInstance, structs::{HostileInfo, Roster}},
};

use super::player::PlayerClass;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RosterItem {
    pub name: String,
    pub unit_id: u32,
    pub player_class: PlayerClass,
    pub player_level: u16,
    pub party_id: u16,
    pub area: LevelName,
    /// raw level id (also for mod-added levels that have no LevelName)
    pub area_id: u32,
    pub pos_x: u32,
    pub pos_y: u32,
    pub party_flags: u32,
    pub hostile_info: Vec<HostileInfo>,
}

impl RosterItem {
    pub fn new(roster: Roster, hostile_info: Vec<HostileInfo>) -> Self {
        RosterItem {
            name: parse_arr_to_string(&roster.name),
            unit_id: roster.dw_unit_id,
            player_class: unsafe { transmute::<u32, PlayerClass>(roster.player_class as u32) },
            player_level: roster.player_level,
            party_id: roster.party_id,
            // a mod-added level id is not a valid LevelName, so never transmute it
            area: num_traits::FromPrimitive::from_u32(roster.area as u32).unwrap_or_default(),
            area_id: roster.area as u32,
            pos_x: roster.pos_x,
            pos_y: roster.pos_y,
            party_flags: roster.party_flags,
            hostile_info,
        }
    }
}

pub fn get_roster(d2rprocess: &D2RInstance) -> Vec<RosterItem> {
    let mut roster_list: Vec<RosterItem> = vec![];
    let roster_ptr = d2rprocess.read_mem_offset::<u64>(d2rprocess.offsets.roster);
    if roster_ptr > 0 {
        follow_p_next(d2rprocess, roster_ptr, &mut roster_list);
    }
    roster_list
}

fn follow_p_next(d2rprocess: &D2RInstance, memory_address: u64, roster_list: &mut Vec<RosterItem>) {
    let roster: Roster = d2rprocess.read_mem::<Roster>(memory_address);
    let mut p_hostile_info: u64 = d2rprocess.read_mem::<u64>(roster.hostile_info);
    let mut hostile_info: Vec<HostileInfo> = vec![];
    while p_hostile_info > 0 {
        let this_hostile_info: HostileInfo = d2rprocess.read_mem::<HostileInfo>(p_hostile_info);
        p_hostile_info = this_hostile_info.next_hostile_info.clone();
        hostile_info.push(this_hostile_info);
    }
    let roster_item = RosterItem::new(roster, hostile_info);
    roster_list.push(roster_item);
    if roster.next_roster > 0 {
        follow_p_next(d2rprocess, roster.next_roster, roster_list);
    }
}

fn parse_arr_to_string(bytes: &[u8]) -> String {
    let mut fixed_string: Vec<u8> = vec![];
    for b in bytes {
        if *b == 0 {
            break;
        }
        fixed_string.push(b.clone());
    }
    unsafe { String::from_utf8_unchecked(fixed_string) }
}