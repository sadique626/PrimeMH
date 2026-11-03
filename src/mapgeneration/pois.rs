use crate::mapgeneration::jsondata::{LevelData, Object};

use super::jsondata::{LevelName, Offset};

use num_traits::FromPrimitive;

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Debug)]
pub struct POI {
    pub id: u32,
    pub poi_type: POIType,
    pub pos_x: f32,
    pub pos_y: f32,
    pub world_x: u32,
    pub world_y: u32,
    pub class: String,
    pub label: String,
}

impl Default for POI {
    fn default() -> Self {
        POI {
            id: 0,
            poi_type: POIType::default(),
            pos_x: 0.0,
            pos_y: 0.0,
            world_x: 0,
            world_y: 0,
            class: String::new(),
            label: String::new(),
        }
    }
}

impl POI {
    pub fn new(obj: &Object, offset: &Offset, poi_type: POIType) -> Self {
        Self::new_label(obj, offset, poi_type, String::new())
    }
    pub fn new_label(obj: &Object, offset: &Offset, poi_type: POIType, label: String) -> Self {
        POI {
            id: obj.id,
            poi_type,
            pos_x: obj.x as f32,
            pos_y: obj.y as f32,
            world_x: obj.x + offset.x,
            world_y: obj.y + offset.y,
            class: String::from(obj.class.as_str()),
            label,
        }
    }
    pub fn new_shrine(x: u32, y: u32, offset: &Offset, label: String) -> Self {
        POI {
            id: 2,
            poi_type: POIType::Shrine,
            pos_x: x as f32 - offset.x as f32,
            pos_y: y as f32 - offset.y as f32,
            world_x: x,
            world_y: y,
            class: "".to_string(),
            label,
        }
    }
    pub fn new_well(x: u32, y: u32, offset: &Offset, label: String) -> Self {
        POI {
            id: 2,
            poi_type: POIType::Well,
            pos_x: x as f32 - offset.x as f32,
            pos_y: y as f32 - offset.y as f32,
            world_x: x,
            world_y: y,
            class: "".to_string(),
            label,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, Default, PartialEq)]
pub enum POIType {
    Waypoint,
    Shrine,
    Well,
    Chest,
    SuperChest,
    Exit,
    GoodExit,
    QuestItem,
    NPCSpawn,
    #[default]
    Unknown,
}

// convert the blacha JSON to usable POI objects
pub fn get_preset_pois(level_data: &LevelData) -> Vec<POI> {
    let mut pois: Vec<POI> = vec![];

    for object in &level_data.objects {
        if object.name == "Waypoint" {
            pois.push(POI::new(object, &level_data.offset, POIType::Waypoint));
        }

        if object.name == "chest" && (level_data.id == 84 || level_data.id == 85 || level_data.id == 91) {
            pois.push(POI::new(object, &level_data.offset, POIType::SuperChest));
        }
        if object.name == "chest" && (level_data.id == 85 || level_data.id == 91) {
            pois.push(POI::new(object, &level_data.offset, POIType::QuestItem));
        }
        if object.class == "chest-super" && object.id == 580 {
            pois.push(POI::new(object, &level_data.offset, POIType::SuperChest));
        }
        if object.name == "Shrine" {
            pois.push(POI::new(object, &level_data.offset, POIType::Shrine));
        }

        if object.name == "Well" {
            pois.push(POI::new(object, &level_data.offset, POIType::Well));
        }

        if object.object_type == "exit" {
            if object.is_good_exit && level_data.id == 46 {
                pois.push(POI::new(object, &level_data.offset, POIType::GoodExit));
            } else {
                let label = LevelName::from_u32(object.id).unwrap_or_default().to_string();
                let exit_poi = POI::new_label(object, &level_data.offset, POIType::Exit, label);
                pois.push(exit_poi);
            }
        }

        if object.object_type == "npc" {
            // summoner
            if level_data.id == 74 && object.id == 250 {
                pois.push(POI::new(object, &level_data.offset, POIType::NPCSpawn));
            }
            // izual
            if level_data.id == 105 && object.object_type == "npc" {
                pois.push(POI::new(object, &level_data.offset, POIType::NPCSpawn));
            }
            // maggot lair 3
            if level_data.id == 64 && object.object_type == "npc" {
                pois.push(POI::new(object, &level_data.offset, POIType::NPCSpawn));
            }
            // radament
            if level_data.id == 49 && object.id == 744 {
                pois.push(POI::new(object, &level_data.offset, POIType::NPCSpawn));
            }
            // nihlithak is calculated by the preset NPC on the _opposite_ side of the map
            if level_data.id == 124 && object.object_type == "npc" {
                let mut x = object.x;
                let mut y = object.y;
                if x == 30 && y == 208 {
                    // bottom right
                    x = 395;
                    y = 210;
                }
                if x == 206 && y == 32 {
                    // bottom left
                    x = 210;
                    y = 395;
                }
                if x == 207 && y == 393 {
                    // top right
                    x = 210;
                    y = 25;
                }
                if x == 388 && y == 216 {
                    //top left
                    x = 25;
                    y = 210;
                }
                let nihl_x = x as u32;
                let nihl_y = y as u32;

                let nihl = Object::new_npc(nihl_x, nihl_y, object.id);

                pois.push(POI::new(&nihl, &level_data.offset, POIType::NPCSpawn));
            }
        }

        // guaranteed super uniques and act bosses, which always spawn at their preset spot
        if object.object_type == "npc" {
            if let Some(name) = fixed_boss_name(object.id) {
                let near = |p: &POI| {
                    p.poi_type == POIType::NPCSpawn
                        && (p.pos_x - object.x as f32).abs() < 6.0
                        && (p.pos_y - object.y as f32).abs() < 6.0
                };
                match pois.iter_mut().find(|p| near(p)) {
                    // one of the special cases above already marked it: just give it the name
                    Some(p) => p.label = name.to_owned(),
                    None => pois.push(POI::new_label(object, &level_data.offset, POIType::NPCSpawn, name.to_owned())),
                }
            }
        }

        // quest items
        if level_data.id != 75 {
            match object.name.as_str() {
                "orifice" | "gidbinn altar" | "Hellforge" | "cagedwussie1" | "Tome" | "LamTome" | "Inifuss"
                | "taintedsunaltar" | "Seal" | "StoneLambda" => {
                    pois.push(POI::new(object, &level_data.offset, POIType::QuestItem));
                }
                _ => (),
            }
        }
        //anya
        if object.object_type == "npc" && level_data.id == 114 {
            pois.push(POI::new(object, &level_data.offset, POIType::QuestItem));
        }
    }
    pois
}

/// SuperUniques.txt, in game order (the "Expansion" divider row removed). In the map data a
/// preset monster id of 734 + index means "this super unique spawns here".
const SUPER_UNIQUES: [&str; 66] = [
    "Bishibosh",
    "Bonebreaker",
    "Coldcrow",
    "Rakanishu",
    "Treehead WoodFist",
    "Griswold",
    "The Countess",
    "Pitspawn Fouldog",
    "Flamespike the Crawler",
    "Bone Ash",
    "Radament",
    "Bloodwitch the Wild",
    "Fangskin",
    "Beetleburst",
    "Creeping Feature",
    "Coldworm the Burrower",
    "Fire Eye",
    "Dark Elder",
    "The Summoner",
    "Ancient Kaa the Soulless",
    "The Smith",
    "Sszark the Burning",
    "Witch Doctor Endugu",
    "Stormtree",
    "Battlemaid Sarina",
    "Icehawk Riftwing",
    "Ismail Vilehand",
    "Geleb Flamefinger",
    "Bremm Sparkfist",
    "Toorc Icefist",
    "Wyand Voidbringer",
    "Maffer Dragonhand",
    "Darkwing",
    "The Tormentor",
    "Taintbreeder",
    "Riftwraith the Cannibal",
    "Infector of Souls",
    "Lord De Seis",
    "Grand Vizier of Chaos",
    "The Cow King",
    "Corpsefire",
    "Hephasto The Armorer",
    "Shenk the Overseer",
    "Talic",
    "Madawc",
    "Korlic",
    "Axe Dweller",
    "Bonesaw Breaker",
    "Dac Farren",
    "Eldritch the Rectifier",
    "Eyeback the Unleashed",
    "Thresh Socket",
    "Pindleskin",
    "Snapchip Shatter",
    "Hell's Belle",
    "Vinvear Molech",
    "Sharptooth Slayer",
    "Magma Torquer",
    "Blaze Ripper",
    "Frozenstein",
    "Nihlathak",
    "Colenzo the Annihilator",
    "Achmel the Cursed",
    "Bartuc the Bloody",
    "Ventar the Unholy",
    "Lister the Tormentor",
];
const FIRST_SUPER_UNIQUE_ID: u32 = 734;

/// Name of a monster that always spawns at its preset spot, or None for ordinary monsters.
pub fn fixed_boss_name(preset_id: u32) -> Option<&'static str> {
    if (FIRST_SUPER_UNIQUE_ID..FIRST_SUPER_UNIQUE_ID + SUPER_UNIQUES.len() as u32).contains(&preset_id) {
        return Some(SUPER_UNIQUES[(preset_id - FIRST_SUPER_UNIQUE_ID) as usize]);
    }
    // act bosses and unique quest monsters placed by MonStats id
    match preset_id {
        156 => Some("Andariel"),
        211 => Some("Duriel"),
        242 => Some("Mephisto"),
        243 => Some("Diablo"),
        250 => Some("The Summoner"),
        256 => Some("Izual"),
        267 => Some("Blood Raven"),
        544 => Some("Baal"),
        _ => None,
    }
}
