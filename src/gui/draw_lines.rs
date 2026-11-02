use notan::draw::*;
use notan::prelude::*;

use crate::mapgeneration::jsondata::LevelData;
use crate::mapgeneration::jsondata::LevelName;

use crate::mapgeneration::pois::POIType;
use crate::memory::gamedata::GameData;
use crate::settings::Settings;

use num_traits::FromPrimitive;

// this will draw the preset POI data from the generated map data
// this includes waypoints, exits, certain shrines, super chests, NPC spawn locations
pub fn draw_lines(draw: &mut Draw, this_level: &LevelData, game_data: &GameData, settings: &Settings, width: &f32, height: &f32) {
    let player_pos = (game_data.player.pos_x, game_data.player.pos_y);
    let current_level_id = game_data.seed_values.level;
    if this_level.id == game_data.seed_values.level && !this_level.is_town() {
        let pois = &this_level.level_image.pois;
        for poi in pois.iter() {
            let preset_pos = (poi.pos_x + this_level.offset.x as f32, poi.pos_y + this_level.offset.y as f32);
            let target_pos = transform_position(preset_pos, player_pos, settings.visual.scale, width, height);
            match poi.poi_type {
                POIType::Waypoint => {
                    draw_waypoint_line(target_pos, draw, settings, width, height);
                }
                POIType::Exit => {
                    draw_exit_line(target_pos, draw, poi.id, current_level_id, settings, width, height);
                }
                POIType::GoodExit => {
                    // line to the real Tal Rasha's tomb in Canyon of the Magi
                    if settings.lines.exit_enabled {
                        draw.line((*width / 2.0, *height / 2.0), target_pos)
                            .width(settings.lines.line_width)
                            .color(Color::from_rgb(0.0, 1.0, 0.0));
                    }
                }
                POIType::QuestItem => {
                    draw_quest_line(target_pos, draw, settings, width, height);
                }
                POIType::NPCSpawn => {
                    draw_boss_spawn_line(target_pos, draw, settings, width, height);
                }
                _ => (),
            }
        }
    }
}

fn draw_waypoint_line(target_pos: (f32, f32), draw: &mut Draw, settings: &Settings, width: &f32, height: &f32) {
    if !settings.lines.waypoint_enabled {
        return;
    }
    let line_color: Color = convert_color(settings.lines.waypoint_rgba);
    draw.line((*width as f32 / 2.0, *height as f32 / 2.0), target_pos)
        .width(settings.lines.line_width)
        .color(line_color);
}

fn draw_quest_line(target_pos: (f32, f32), draw: &mut Draw, settings: &Settings, width: &f32, height: &f32) {
    if !settings.lines.quest_enabled {
        return;
    }
    let line_color: Color = convert_color(settings.lines.quest_rgba);
    draw.line((*width as f32 / 2.0, *height as f32 / 2.0), target_pos)
        .color(line_color)
        .width(settings.lines.line_width)
        .alpha(0.5);
}

fn draw_boss_spawn_line(target_pos: (f32, f32), draw: &mut Draw, settings: &Settings, width: &f32, height: &f32) {
    if !settings.lines.boss_enabled {
        return;
    }
    let line_color: Color = convert_color(settings.lines.boss_rgba);
    draw.line((*width as f32 / 2.0, *height as f32 / 2.0), target_pos)
        .color(line_color)
        .width(settings.lines.line_width)
        .alpha(0.5);
}

fn draw_exit_line(
    target_pos: (f32, f32),
    draw: &mut Draw,
    target_exit_id: u32,
    current_level_id: u32,
    settings: &Settings,
    width: &f32, 
    height: &f32
) {
    if !settings.lines.exit_enabled {
        return;
    }
    let line_color: Color = convert_color(settings.lines.exit_rgba);
    let current_level_name = LevelName::from_u32(current_level_id).unwrap_or_default();
    let target_exit_name = LevelName::from_u32(target_exit_id).unwrap_or_default();
    let exits = get_next_exits(current_level_name);
    if exits.contains(&target_exit_name) {
        draw.line((*width as f32 / 2.0, *height as f32 / 2.0), target_pos)
            .width(settings.lines.line_width)
            .color(line_color);
    }
}

fn convert_color(color_arr: [u8; 4]) -> Color {
    Color::from_bytes(color_arr[0], color_arr[1], color_arr[2], color_arr[3])
}

fn transform_position(unit_pos: (f32, f32), player_pos: (f32, f32), scale: f32, width: &f32, height: &f32) -> (f32, f32) {
    let xdiff = unit_pos.0 - player_pos.0;
    let ydiff = unit_pos.1 - player_pos.1;

    let center_x = *width as f32 / 2.0;
    let center_y = *height as f32 / 2.0;
    let angle: f32 = std::f32::consts::FRAC_PI_4;
    let x = xdiff * angle.cos() - ydiff * angle.sin();
    let y = xdiff * angle.sin() + ydiff * angle.cos();

    let new_pos_x = center_x + (x * scale);
    let new_pos_y = center_y + (y * scale * 0.5);

    (new_pos_x, new_pos_y)
}

pub fn get_next_exits(current_level_name: LevelName) -> Vec<LevelName> {
    match current_level_name {
        LevelName::None => vec![],
        LevelName::RogueEncampment => vec![],
        LevelName::BloodMoor => vec![LevelName::DenOfEvil, LevelName::ColdPlains],
        LevelName::ColdPlains => vec![LevelName::StonyField, LevelName::BurialGrounds],
        LevelName::StonyField => vec![LevelName::UndergroundPassageLevel1],
        LevelName::DarkWood => vec![LevelName::BlackMarsh],
        LevelName::BlackMarsh => vec![LevelName::TamoeHighland, LevelName::ForgottenTower],
        LevelName::TamoeHighland => vec![LevelName::PitLevel1],
        LevelName::DenOfEvil => vec![LevelName::BloodMoor],
        LevelName::CaveLevel1 => vec![LevelName::CaveLevel2],
        LevelName::UndergroundPassageLevel1 => vec![LevelName::DarkWood],
        LevelName::HoleLevel1 => vec![LevelName::HoleLevel2],
        LevelName::PitLevel1 => vec![LevelName::PitLevel2],
        LevelName::CaveLevel2 => vec![],
        LevelName::UndergroundPassageLevel2 => vec![],
        LevelName::HoleLevel2 => vec![],
        LevelName::PitLevel2 => vec![],
        LevelName::BurialGrounds => vec![],
        LevelName::Crypt => vec![],
        LevelName::Mausoleum => vec![],
        LevelName::ForgottenTower => vec![],
        LevelName::TowerCellarLevel1 => vec![LevelName::TowerCellarLevel2],
        LevelName::TowerCellarLevel2 => vec![LevelName::TowerCellarLevel3],
        LevelName::TowerCellarLevel3 => vec![LevelName::TowerCellarLevel4],
        LevelName::TowerCellarLevel4 => vec![LevelName::TowerCellarLevel5],
        LevelName::TowerCellarLevel5 => vec![],
        LevelName::MonasteryGate => vec![],
        LevelName::OuterCloister => vec![LevelName::Barracks],
        LevelName::Barracks => vec![LevelName::JailLevel1],
        LevelName::JailLevel1 => vec![LevelName::JailLevel2],
        LevelName::JailLevel2 => vec![LevelName::JailLevel3],
        LevelName::JailLevel3 => vec![LevelName::InnerCloister],
        LevelName::InnerCloister => vec![LevelName::Cathedral],
        LevelName::Cathedral => vec![LevelName::CatacombsLevel1],
        LevelName::CatacombsLevel1 => vec![LevelName::CatacombsLevel2],
        LevelName::CatacombsLevel2 => vec![LevelName::CatacombsLevel3],
        LevelName::CatacombsLevel3 => vec![LevelName::CatacombsLevel4],
        LevelName::CatacombsLevel4 => vec![],
        LevelName::Tristram => vec![],
        LevelName::SecretCowLevel => vec![],
        LevelName::LutGholein => vec![],
        LevelName::RockyWaste => vec![LevelName::DryHills],
        LevelName::DryHills => vec![LevelName::FarOasis, LevelName::HallsOfTheDeadLevel1],
        LevelName::FarOasis => vec![LevelName::LostCity, LevelName::MaggotLairLevel1],
        LevelName::LostCity => vec![LevelName::ValleyOfSnakes, LevelName::AncientTunnels],
        LevelName::ValleyOfSnakes => vec![LevelName::ClawViperTempleLevel1],
        LevelName::CanyonOfTheMagi => vec![],
        LevelName::SewersLevel1 => vec![LevelName::SewersLevel2],
        LevelName::SewersLevel2 => vec![LevelName::SewersLevel3],
        LevelName::SewersLevel3 => vec![],
        LevelName::HaremLevel1 => vec![],
        LevelName::HaremLevel2 => vec![],
        LevelName::PalaceCellarLevel1 => vec![],
        LevelName::PalaceCellarLevel2 => vec![],
        LevelName::PalaceCellarLevel3 => vec![],
        LevelName::StonyTombLevel1 => vec![LevelName::StonyTombLevel2],
        LevelName::HallsOfTheDeadLevel1 => vec![LevelName::HallsOfTheDeadLevel2],
        LevelName::HallsOfTheDeadLevel2 => vec![LevelName::HallsOfTheDeadLevel3],
        LevelName::ClawViperTempleLevel1 => vec![LevelName::ClawViperTempleLevel2],
        LevelName::StonyTombLevel2 => vec![],
        LevelName::HallsOfTheDeadLevel3 => vec![],
        LevelName::ClawViperTempleLevel2 => vec![],
        LevelName::MaggotLairLevel1 => vec![LevelName::MaggotLairLevel2],
        LevelName::MaggotLairLevel2 => vec![LevelName::MaggotLairLevel3],
        LevelName::MaggotLairLevel3 => vec![],
        LevelName::AncientTunnels => vec![],
        LevelName::TalRashasTombStar => vec![],
        LevelName::TalRashasTombSquare => vec![],
        LevelName::TalRashasTombSemiCircle => vec![],
        LevelName::TalRashasTombCircle => vec![],
        LevelName::TalRashasTombTwoChevrons => vec![],
        LevelName::TalRashasTombTriangle => vec![],
        LevelName::TalRashasTombCirclewithline => vec![],
        LevelName::DurielsLair => vec![],
        LevelName::ArcaneSanctuary => vec![],
        LevelName::KurastDocks => vec![],
        LevelName::SpiderForest => vec![LevelName::SpiderCavern],
        LevelName::GreatMarsh => vec![],
        LevelName::FlayerJungle => vec![LevelName::FlayerDungeonLevel1],
        LevelName::LowerKurast => vec![LevelName::KurastBazaar],
        LevelName::KurastBazaar => vec![LevelName::UpperKurast],
        LevelName::UpperKurast => vec![LevelName::KurastCauseway],
        LevelName::KurastCauseway => vec![],
        LevelName::Travincal => vec![LevelName::DuranceOfHateLevel1],
        LevelName::ArachnidLair => vec![],
        LevelName::SpiderCavern => vec![],
        LevelName::SwampyPitLevel1 => vec![LevelName::SwampyPitLevel2],
        LevelName::SwampyPitLevel2 => vec![LevelName::SwampyPitLevel3],
        LevelName::FlayerDungeonLevel1 => vec![LevelName::FlayerDungeonLevel2],
        LevelName::FlayerDungeonLevel2 => vec![LevelName::FlayerDungeonLevel3],
        LevelName::SwampyPitLevel3 => vec![],
        LevelName::FlayerDungeonLevel3 => vec![],
        LevelName::Act3SewersLevel1 => vec![LevelName::Act3SewersLevel2],
        LevelName::Act3SewersLevel2 => vec![],
        LevelName::RuinedTemple => vec![],
        LevelName::DisusedFane => vec![],
        LevelName::ForgottenReliquary => vec![],
        LevelName::ForgottenTemple => vec![],
        LevelName::RuinedFane => vec![],
        LevelName::DisusedReliquary => vec![],
        LevelName::DuranceOfHateLevel1 => vec![LevelName::DuranceOfHateLevel2],
        LevelName::DuranceOfHateLevel2 => vec![LevelName::DuranceOfHateLevel3],
        LevelName::DuranceOfHateLevel3 => vec![],
        LevelName::PandemoniumFortress => vec![],
        LevelName::OuterSteppes => vec![LevelName::PlainsOfDespair],
        LevelName::PlainsOfDespair => vec![LevelName::CityOfTheDamned],
        LevelName::CityOfTheDamned => vec![LevelName::RiverOfFlame],
        LevelName::RiverOfFlame => vec![LevelName::ChaosSanctuary],
        LevelName::ChaosSanctuary => vec![],
        LevelName::Harrogath => vec![],
        LevelName::BloodyFoothills => vec![],
        LevelName::FrigidHighlands => vec![],
        LevelName::ArreatPlateau => vec![],
        LevelName::CrystallinePassage => vec![LevelName::FrozenRiver],
        LevelName::FrozenRiver => vec![],
        LevelName::GlacialTrail => vec![LevelName::FrozenTundra],
        LevelName::DrifterCavern => vec![],
        LevelName::FrozenTundra => vec![],
        LevelName::AncientsWay => vec![LevelName::ArreatSummit],
        LevelName::IcyCellar => vec![],
        LevelName::ArreatSummit => vec![],
        LevelName::NihlathaksTemple => vec![],
        LevelName::HallsOfAnguish => vec![LevelName::HallsOfPain],
        LevelName::HallsOfPain => vec![LevelName::HallsOfVaught],
        LevelName::HallsOfVaught => vec![],
        LevelName::Abaddon => vec![],
        LevelName::PitOfAcheron => vec![],
        LevelName::InfernalPit => vec![],
        LevelName::WorldstoneKeepLevel1 => vec![LevelName::WorldstoneKeepLevel2],
        LevelName::WorldstoneKeepLevel2 => vec![LevelName::WorldstoneKeepLevel3],
        LevelName::WorldstoneKeepLevel3 => vec![LevelName::ThroneOfDestruction],
        LevelName::ThroneOfDestruction => vec![],
        LevelName::WorldstoneChamber => vec![],
        LevelName::MatronsDen => vec![],
        LevelName::ForgottenSands => vec![],
        LevelName::FurnaceOfPain => vec![],
        LevelName::UberTristram => vec![],
    }
}
