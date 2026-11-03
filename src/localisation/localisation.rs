#[allow(non_snake_case)]
use std::collections::HashMap;
use regex::Regex;
use notan::text::Font;


use crate::{gui::Fonts, settings::Locales};

use super::localisation_file_parser::{load_localisation_files, LocalisationFiles, LocalisationRawFileEntry};

 

pub struct LocalisationLanguage {
    pub item_gems: HashMap<String, String>,
    pub item_modifiers: HashMap<String, String>,
    pub item_nameaffixes: HashMap<String, String>,
    pub item_names: HashMap<String, String>,
    pub item_runes: HashMap<String, String>,
    pub levels: HashMap<u32, String>,
    pub mercenaries: HashMap<String, String>,
    pub monsters: HashMap<String, String>,
    pub npcs: HashMap<String, String>,
    pub objects: HashMap<String, String>,
    pub quests: HashMap<String, String>,
    pub shrines: HashMap<u32, String>,
    pub skills: HashMap<String, String>,
    pub primemh: HashMap<String, String>,
}

impl LocalisationLanguage {
    pub fn new(localisation_source_files: &HashMap<LocalisationFiles, Vec<LocalisationRawFileEntry>>, locale: &Locales) -> Self {
        let item_gems: HashMap<String, String> = vec_to_hashmap(localisation_source_files.get(&LocalisationFiles::ItemGemsData).unwrap(), locale);
        let item_modifiers: HashMap<String, String> = vec_to_hashmap(localisation_source_files.get(&LocalisationFiles::ItemModifiersData).unwrap(), locale);
        let item_nameaffixes: HashMap<String, String> = vec_to_hashmap(localisation_source_files.get(&LocalisationFiles::ItemNameAffixesData).unwrap(), locale);
        let item_names: HashMap<String, String> = vec_to_hashmap(localisation_source_files.get(&LocalisationFiles::ItemNamesData).unwrap(), locale);
        let item_runes: HashMap<String, String> = vec_to_hashmap(localisation_source_files.get(&LocalisationFiles::ItemRunesData).unwrap(), locale);
        let levels: HashMap<u32, String> = vec_to_id_hashmap(localisation_source_files.get(&LocalisationFiles::LevelsData).unwrap(), locale);
        let mercenaries: HashMap<String, String> = vec_to_hashmap(localisation_source_files.get(&LocalisationFiles::MercData).unwrap(), locale);
        let monsters: HashMap<String, String> = vec_to_hashmap(localisation_source_files.get(&LocalisationFiles::MonstersData).unwrap(), locale);
        let npcs: HashMap<String, String> = vec_to_hashmap(localisation_source_files.get(&LocalisationFiles::NPCsData).unwrap(), locale);
        let objects: HashMap<String, String> = vec_to_hashmap(localisation_source_files.get(&LocalisationFiles::ObjectsData).unwrap(), locale);
        let quests: HashMap<String, String> = vec_to_hashmap(localisation_source_files.get(&LocalisationFiles::QuestsData).unwrap(), locale);
        let shrines: HashMap<u32, String> = vec_to_id_hashmap(localisation_source_files.get(&LocalisationFiles::ShrinesData).unwrap(), locale);
        let skills: HashMap<String, String> = vec_to_hashmap(localisation_source_files.get(&LocalisationFiles::SkillsData).unwrap(), locale);
        let primemh: HashMap<String, String> = vec_to_hashmap(localisation_source_files.get(&LocalisationFiles::PrimeMHData).unwrap(), locale);
        LocalisationLanguage { item_gems, item_modifiers, item_nameaffixes, item_names, item_runes, levels, mercenaries, monsters, npcs, objects, quests, shrines, skills, primemh }
    }
    pub fn get_primemh(&self, key_name: &'static str) -> String {
        let new_string = self.primemh.get(key_name);
        return match new_string {
            Some(s) => s.clone(),
            None => String::from(key_name),
        }
    }

    pub fn get_item_name(&self, key_name: &String) -> String {
        let new_key_name: String = key_name.chars()
            .filter(|&c| c != '\'' && c != '-' && !c.is_whitespace())
            .collect::<String>()
            .to_lowercase();
        let new_string = self.item_names.get(&new_key_name);
        let new_string2 = self.item_runes.get(&new_key_name);
        let new_string3 = self.item_gems.get(&new_key_name);

        return match (new_string, new_string2, new_string3) {
            (Some(value), None, None) => value.clone(),
            (None, Some(value), None) => value.clone(),
            (None, None, Some(value)) => value.clone(),
            _ => String::from(new_key_name),
        }
    }

    pub fn get_npc_name(&self, key_name: &String) -> String {
        let new_key_name: String = key_name.chars()
            .filter(|&c| !c.is_digit(10) && !c.is_whitespace())
            .collect();
        let new_string = self.npcs.get(&new_key_name.to_lowercase().replace("-",""));
        return match new_string {
            Some(s) => s.clone(),
            None => key_name.clone(),
        }
    }

    pub fn get_shrine(&self, interact_type: usize) -> String {
        if interact_type > 22 || interact_type < 1 {
            log::debug!("Error getting shrine name for {:?}", interact_type);
        }
        // map shrine interact_type with localisation file id
        let localised_id: u32 = match interact_type {
            1 => 10810,
            2 => 10811,
            3 => 10812,
            4 => 10813,
            5 => 10814,
            6 => 10815,
            7 => 10816,
            8 => 10817,
            9 => 10818,
            10 => 10819,
            11 => 10820,
            12 => 10821,
            13 => 10822,
            14 => 10823,
            15 => 10824,
            16 => 10825,
            17 => 10826,
            18 => 10827,
            19 => 10828,
            20 => 10829,
            21 => 10830,
            22 => 10831,
            _ => 0,
        };
        match self.shrines.get(&localised_id) {
            Some(s) => {
                s.to_string()
            },
            None => {
                log::debug!("ERROR: Shrine interact type {:?} localised id {:?}", interact_type, localised_id);
                String::new()
            },
        }
    }

    pub fn get_level(&self, txt_file_no: &u32) -> String {
        if txt_file_no > &136 || txt_file_no < &1 {
            log::debug!("Error getting level name for {:?}", txt_file_no);
        }
        let localised_id = match txt_file_no {
            1 => 5055,
            2 => 5054,
            3 => 5053,
            4 => 5052,
            5 => 5051,
            6 => 5050,
            7 => 5049,
            8 => 5048,
            9 => 5047,
            10 => 5046,
            11 => 5045,
            12 => 5044,
            13 => 5043,
            14 => 5042,
            15 => 5041,
            16 => 5040,
            17 => 5039,
            18 => 5038,
            19 => 5037,
            20 => 5036,
            21 => 5035,
            22 => 5034,
            23 => 5033,
            24 => 5032,
            25 => 5031,
            26 => 5030,
            27 => 5029,
            28 => 5028,
            29 => 5027,
            30 => 5026,
            31 => 5025,
            32 => 5024,
            33 => 5023,
            34 => 5022,
            35 => 5021,
            36 => 5020,
            37 => 5019,
            38 => 5018,
            39 => 788,
            40 => 852,
            41 => 851,
            42 => 850,
            43 => 849,
            44 => 848,
            45 => 847,
            46 => 846,
            47 => 845,
            48 => 844,
            49 => 843,
            50 => 842,
            51 => 841,
            52 => 840,
            53 => 839,
            54 => 838,
            55 => 837,
            56 => 836,
            57 => 835,
            58 => 834,
            59 => 833,
            60 => 832,
            61 => 831,
            62 => 830,
            63 => 829,
            64 => 828,
            65 => 827,
            66 => 826,
            67 => 826,
            68 => 826,
            69 => 826,
            70 => 826,
            71 => 826,
            72 => 826,
            73 => 825,
            74 => 824,
            75 => 820,
            76 => 819,
            77 => 818,
            78 => 817,
            79 => 816,
            80 => 815,
            81 => 814,
            82 => 813,
            83 => 812,
            84 => 810,
            85 => 811,
            86 => 809,
            87 => 808,
            88 => 806,
            89 => 805,
            90 => 807,
            91 => 804,
            92 => 845,
            93 => 844,
            94 => 803,
            95 => 802,
            96 => 801,
            97 => 800,
            98 => 799,
            99 => 798,
            100 => 797,
            101 => 796,
            102 => 795,
            103 => 790,
            104 => 792,
            105 => 793,
            106 => 794,
            107 => 791,
            108 => 789,
            109 => 22646,
            110 => 22647,
            111 => 22648,
            112 => 22649,
            113 => 22650,
            114 => 22651,
            115 => 22652,
            116 => 22653,
            117 => 22654,
            118 => 22655,
            119 => 22656,
            120 => 22657,
            121 => 22658,
            122 => 22659,
            123 => 22660,
            124 => 22662,
            125 => 21865,
            126 => 21866,
            127 => 21867,
            128 => 22663,
            129 => 22664,
            130 => 22665,
            131 => 22667,
            132 => 22666,
            133 => 11155,
            134 => 11156,
            135 => 11157,
            136 => 5018,
            _ => 0
        };
        match self.levels.get(&localised_id) {
            Some(s) => {
                s.to_string()
            },
            None => {
                log::debug!("ERROR: txt_file_no {:?} localised id {:?}", txt_file_no, localised_id);
                String::new()
            },
        }
    }
}


pub struct Localisation {
    pub languages: HashMap<Locales, LocalisationLanguage>,
    pub current_locale: Locales
}

impl Localisation {
    pub fn new() -> Self {
        Self::initialize()
    }

    pub fn update_locale(&mut self, locale: Locales) {
        self.current_locale = locale;
    }

    pub fn get_primemh(&self, key_name: &'static str) -> String {
        self.languages.get(&self.current_locale).unwrap().get_primemh(key_name)
    }

    pub fn get_item_name(&self, key_name: &String) -> String {
        self.languages.get(&self.current_locale).unwrap().get_item_name(key_name)
    }

    pub fn get_npc_name(&self, key_name: &String) -> String {
        self.languages.get(&self.current_locale).unwrap().get_npc_name(key_name)
    }

    pub fn get_shrine(&self, interact_type: usize) -> String {
        self.languages.get(&self.current_locale).unwrap().get_shrine(interact_type)
    }

    pub fn get_level(&self, txt_file_no: &u32) -> String {
        self.languages.get(&self.current_locale).unwrap().get_level(txt_file_no)
    }

    pub fn initialize() -> Localisation {
        let localisation_source_files: HashMap<LocalisationFiles, Vec<LocalisationRawFileEntry>> = load_localisation_files();
            
        let mut languages: HashMap<Locales, LocalisationLanguage> = HashMap::new();
        languages.insert(Locales::enUS, LocalisationLanguage::new(&localisation_source_files, &Locales::enUS));
        languages.insert(Locales::zhTW, LocalisationLanguage::new(&localisation_source_files, &Locales::zhTW));
        languages.insert(Locales::deDE, LocalisationLanguage::new(&localisation_source_files, &Locales::deDE));
        languages.insert(Locales::esES, LocalisationLanguage::new(&localisation_source_files, &Locales::esES));
        languages.insert(Locales::frFR, LocalisationLanguage::new(&localisation_source_files, &Locales::frFR));
        languages.insert(Locales::itIT, LocalisationLanguage::new(&localisation_source_files, &Locales::itIT));
        languages.insert(Locales::koKR, LocalisationLanguage::new(&localisation_source_files, &Locales::koKR));
        languages.insert(Locales::plPL, LocalisationLanguage::new(&localisation_source_files, &Locales::plPL));
        languages.insert(Locales::enBG, LocalisationLanguage::new(&localisation_source_files, &Locales::enBG));
        
        Localisation { languages, current_locale: Locales::enUS }
    }
}



pub fn vec_to_hashmap(file_data: &Vec<LocalisationRawFileEntry>, locale: &Locales) -> HashMap<String, String> {
    let mut hashmap: HashMap<String, String> = HashMap::new();
    for entry in file_data {
        
        let val: String = match locale {
            Locales::enUS => entry.enUS.clone(),
            Locales::zhTW => entry.zhTW.clone(),
            Locales::deDE => entry.deDE.clone(),
            Locales::esES => entry.esES.clone(),
            Locales::frFR => entry.frFR.clone(),
            Locales::itIT => entry.itIT.clone(),
            Locales::koKR => entry.koKR.clone(),
            Locales::plPL => entry.plPL.clone(),
            Locales::enBG => entry.enUS.clone(),
            Locales::Unknown => entry.enUS.clone(),
        };
        let new_key_name: String = entry.Key.chars()
            .filter(|&c| !c.is_whitespace())
            .collect();
        hashmap.insert(new_key_name.to_lowercase().replace("-","").replace("\'",""), val);
    }
    hashmap
}


pub fn vec_to_id_hashmap(file_data: &Vec<LocalisationRawFileEntry>, locale: &Locales) -> HashMap<u32, String> {
    let mut hashmap: HashMap<u32, String> = HashMap::new();
    for entry in file_data {
        
        let val: String = match locale {
            Locales::enUS => entry.enUS.clone(),
            Locales::zhTW => entry.zhTW.clone(),
            Locales::deDE => entry.deDE.clone(),
            Locales::esES => entry.esES.clone(),
            Locales::frFR => entry.frFR.clone(),
            Locales::itIT => entry.itIT.clone(),
            Locales::koKR => entry.koKR.clone(),
            Locales::plPL => entry.plPL.clone(),
            Locales::enBG => entry.enUS.clone(),
            Locales::Unknown => entry.enUS.clone(),
        };
        hashmap.insert(entry.id, val);
    }
    hashmap
}



pub fn detect_safe_font(player_name: String, all_fonts: &Fonts) -> Option<&Font> {
    let chinese_regex = Regex::new("\\p{Han}").unwrap();
    let japanese_regex = Regex::new("\\p{Hiragana}|\\p{Katakana}").unwrap();
    let russian_regex = Regex::new("\\p{Cyrillic}").unwrap();
    let korean_regex = Regex::new("\\p{Hangul}").unwrap();

    if chinese_regex.is_match(&player_name) {
        Some(&all_fonts.taiwan_font)
    } else if japanese_regex.is_match(&player_name) {
        None
    } else if korean_regex.is_match(&player_name) {
        Some(&all_fonts.korean_font)
    } else if russian_regex.is_match(&player_name) {
        None
    } else {
        if player_name.chars().all(|x| x.is_ascii()) {
            Some(&all_fonts.formal_font)
        } else {
            None
        }
    }
}