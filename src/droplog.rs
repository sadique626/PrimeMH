//! Drop log: a list on the right side of the screen of unique and set items that dropped,
//! named even before they are identified (the game stores which unique / set an item is as
//! soon as it drops). Names come from the Reimagined tables the build script downloads into
//! reimagined-data/ (uniqueitems.txt, setitems.txt, strings/enUS.json).

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use notan::draw::*;
use notan::prelude::*;

use crate::gui::Fonts;
use crate::memory::gamedata::{GameData, GroundItem};
use crate::settings::Settings;

const DIR: &str = "reimagined-data";
const MAX_ENTRIES: usize = 14;

struct Drop {
    unit_id: u32,
    name: String,
    base: String,
    set: bool,
    watched: bool,
    at: Instant,
}

/// One row of the filter list in the settings panel.
pub struct CatalogItem {
    pub name: String,
    pub base: String,
    pub set: bool,
    /// lower-case "name base" for searching
    pub search: String,
}

pub struct DropLog {
    /// *ID -> (name key, base item code)
    uniques: HashMap<u32, (String, String)>,
    sets: HashMap<u32, (String, String)>,
    strings: HashMap<String, String>,
    entries: Vec<Drop>,
    seen: HashSet<u32>,
    seed: u32,
    /// every unique and set item, sorted by name, for the filter list
    pub catalog: Vec<CatalogItem>,
    /// text typed into the filter's search box
    pub search: String,
}

pub fn test_sound(file: &str, volume: f32) {
    play_sound(file.to_string(), volume);
}

fn play_sound(file: String, volume: f32) {
    std::thread::spawn(move || {
        let Ok((_stream, handle)) = rodio::OutputStream::try_default() else {
            log::warn!("Drop log: no audio output device");
            return;
        };
        let Ok(sink) = rodio::Sink::try_new(&handle) else { return };
        match std::fs::File::open(&file) {
            Ok(f) => match rodio::Decoder::new(std::io::BufReader::new(f)) {
                Ok(src) => {
                    sink.set_volume(volume.clamp(0.0, 2.0));
                    sink.append(src);
                    sink.sleep_until_end();
                }
                Err(e) => log::warn!("Drop log: could not decode sound {}: {}", file, e),
            },
            Err(e) => log::warn!("Drop log: sound file {} not found: {}", file, e),
        }
    });
}

fn table(file: &str, base_col: &str) -> HashMap<u32, (String, String)> {
    let Ok(bytes) = std::fs::read(std::path::Path::new(DIR).join(file)) else {
        log::warn!("Drop log: {}/{} not found - run the build script to download it", DIR, file);
        return HashMap::new();
    };
    let text: String = bytes.iter().map(|b| *b as char).collect(); // latin-1
    let mut lines = text.lines();
    let header: Vec<&str> = lines.next().unwrap_or("").split('\t').collect();
    let id_col = header.iter().position(|h| *h == "*ID").unwrap_or(1);
    let base = header.iter().position(|h| *h == base_col);
    let mut out = HashMap::new();
    for l in lines {
        let c: Vec<&str> = l.split('\t').collect();
        if let Some(id) = c.get(id_col).and_then(|v| v.trim().parse::<u32>().ok()) {
            let code = base.and_then(|b| c.get(b)).unwrap_or(&"").trim().to_string();
            out.insert(id, (c[0].to_string(), code));
        }
    }
    out
}

impl Default for DropLog {
    fn default() -> Self {
        let strings: HashMap<String, String> = std::fs::read_to_string(std::path::Path::new(DIR).join("strings").join("enUS.json"))
            .ok()
            .and_then(|t| serde_json::from_str::<HashMap<String, serde_json::Value>>(&t).ok())
            .map(|m| m.into_iter().filter_map(|(k, v)| v.as_str().map(|s| (k, s.to_string()))).collect())
            .unwrap_or_default();
        let mut log = DropLog {
            uniques: table("uniqueitems.txt", "code"),
            sets: table("setitems.txt", "item"),
            strings,
            entries: vec![],
            seen: HashSet::new(),
            seed: 0,
            catalog: vec![],
            search: String::new(),
        };
        log.build_catalog();
        log::info!("Drop log: {} uniques, {} set items, {} strings loaded", log.uniques.len(), log.sets.len(), log.strings.len());
        log
    }
}

impl DropLog {
    fn build_catalog(&mut self) {
        let mut seen_names = HashSet::new();
        let mut cat = vec![];
        for (set, map) in [(false, &self.uniques), (true, &self.sets)] {
            for (key, code) in map.values() {
                let name = self.text(key);
                if name.trim().is_empty() || !seen_names.insert(name.clone()) {
                    continue;
                }
                let base = if code.is_empty() { String::new() } else { self.text(code) };
                let search = format!("{} {}", name, base).to_lowercase();
                cat.push(CatalogItem { name, base, set, search });
            }
        }
        cat.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        self.catalog = cat;
    }

    fn text(&self, key: &str) -> String {
        let s = self.strings.get(key).cloned().unwrap_or_else(|| key.to_string());
        // strip a gender tag like "[ms]" and D2 colour codes
        let s = if s.starts_with('[') && s.len() > 4 { s[4..].split('[').next().unwrap_or("").to_string() } else { s };
        s.replace("ÿc", "")
    }

    pub fn update(&mut self, game_data: Option<&GameData>, settings: &Settings) {
        if !settings.drop_log.enabled {
            return;
        }
        let Some(gd) = game_data else { return };
        if gd.seed_values.map_seed != self.seed {
            self.seed = gd.seed_values.map_seed;
            self.seen.clear();
        }
        for item in gd.ground_items.iter() {
            self.add(item, settings);
        }
        let keep = Duration::from_secs(settings.drop_log.duration_secs.max(1) as u64);
        self.entries.retain(|e| e.at.elapsed() < keep);
    }

    fn add(&mut self, item: &GroundItem, settings: &Settings) {
        // 5 = set, 7 = unique
        let set = match item.quality {
            5 => true,
            7 => false,
            _ => return,
        };
        if !self.seen.insert(item.unit_id) {
            return;
        }
        let found = if set { self.sets.get(&item.file_index) } else { self.uniques.get(&item.file_index) };
        let (name, base) = match found {
            Some((key, code)) => (self.text(key), if code.is_empty() { String::new() } else { self.text(code) }),
            None => (format!("{} #{}", if set { "Set item" } else { "Unique" }, item.file_index), String::new()),
        };
        let dl = &settings.drop_log;
        let watched = dl.watch.iter().any(|w| *w == name);
        log::info!("Drop: {} ({}) at {},{}{}", name, base, item.x, item.y, if watched { " [filter]" } else { "" });
        if watched && dl.sound {
            play_sound(dl.sound_file.clone(), dl.volume);
        }
        if dl.only_watched && !watched {
            return;
        }
        self.entries.insert(0, Drop { unit_id: item.unit_id, name, base, set, watched, at: Instant::now() });
        self.entries.truncate(MAX_ENTRIES);
    }

    pub fn draw(&self, draw: &mut Draw, game_data: &GameData, settings: &Settings, fonts: &Fonts, width: f32, height: f32) {
        if !settings.drop_log.enabled || self.entries.is_empty() {
            return;
        }
        let font = fonts.get_safe_font(&settings.general.language);
        let size = settings.drop_log.text_size;
        let x = width * 0.99;
        let mut y = height * settings.drop_log.pos_y;
        let on_ground: HashSet<u32> = game_data.ground_items.iter().map(|i| i.unit_id).collect();
        for e in self.entries.iter() {
            // dim the ones that have been picked up (no longer on the ground)
            let a = if on_ground.contains(&e.unit_id) { 1.0 } else { 0.45 };
            let colour = if e.set { Color::from_rgba(0.30, 0.76, 0.22, a) } else { Color::from_rgba(0.78, 0.72, 0.56, a) };
            let mut line = if e.base.is_empty() { e.name.clone() } else { format!("{}  ({})", e.name, e.base) };
            if e.watched {
                line = format!("> {} <", line);
            }
            draw.text(font, &line).position(x + 1.5, y + 1.5).size(size).color(Color::from_rgba(0.0, 0.0, 0.0, a)).h_align_right().v_align_top();
            draw.text(font, &line).position(x, y).size(size).color(colour).h_align_right().v_align_top();
            y += size * 1.35;
        }
    }
}
