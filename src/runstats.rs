//! Run timer and stats: one "run" is one game (a new map seed). Totals are kept per character
//! in run_stats.json next to the exe, so they survive leaving games and restarting PrimeMH.
//! Switching to a different character starts a fresh set of totals; the settings panel has a
//! reset button too.

use std::collections::HashSet;
use std::time::Instant;

use notan::draw::*;
use notan::prelude::*;
use serde::{Deserialize, Serialize};

use crate::gui::Fonts;
use crate::memory::gamedata::GameData;
use crate::settings::Settings;
use crate::types::npc::{NPCMode, NPCType};
use crate::types::stats::StatEnum;

const FILE: &str = "run_stats.json";
/// games shorter than this (e.g. a quick town visit) don't count as a run
const MIN_RUN_SECS: f64 = 10.0;

#[derive(Serialize, Deserialize, Default, Clone)]
struct Totals {
    character: String,
    runs: u32,
    total_secs: f64,
    best_secs: Option<f64>,
    last_secs: Option<f64>,
    kills: u64,
    xp: u64,
}

struct Run {
    seed: u32,
    started: Instant,
    start_xp: Option<u32>,
    xp_now: Option<u32>,
    dead: HashSet<u32>,
}

pub struct RunStats {
    totals: Totals,
    run: Option<Run>,
}

impl Default for RunStats {
    fn default() -> Self {
        let totals = std::fs::read_to_string(FILE)
            .ok()
            .and_then(|t| serde_json::from_str::<Totals>(&t).ok())
            .unwrap_or_default();
        RunStats { totals, run: None }
    }
}

fn experience(game_data: &GameData) -> Option<u32> {
    if game_data.player_experience.is_some() {
        return game_data.player_experience;
    }
    game_data
        .player
        .stats
        .iter()
        .find(|s| matches!(s.stat, StatEnum::Experience))
        .map(|s| ((s.value2 as u16 as u32) << 16) | (s.value as u16 as u32))
}

fn fmt_time(secs: f64) -> String {
    let s = secs.max(0.0) as u64;
    if s >= 3600 {
        format!("{}:{:02}:{:02}", s / 3600, (s / 60) % 60, s % 60)
    } else {
        format!("{}:{:02}", s / 60, s % 60)
    }
}

fn fmt_num(n: f64) -> String {
    if n >= 1_000_000.0 {
        format!("{:.1}M", n / 1_000_000.0)
    } else if n >= 1_000.0 {
        format!("{:.1}k", n / 1_000.0)
    } else {
        format!("{:.0}", n)
    }
}

impl RunStats {
    pub fn reset(&mut self) {
        let character = self.totals.character.clone();
        self.totals = Totals { character, ..Default::default() };
        if let Some(run) = self.run.as_mut() {
            run.started = Instant::now();
            run.start_xp = run.xp_now;
            run.dead.clear();
        }
        self.save();
    }

    fn save(&self) {
        if let Ok(text) = serde_json::to_string(&self.totals) {
            let _ = std::fs::write(FILE, text);
        }
    }

    /// Call every frame with the current game (None while in the menus).
    pub fn update(&mut self, game_data: Option<&GameData>) {
        let Some(gd) = game_data else {
            self.finish_run();
            return;
        };
        let name = gd.player.player_name.clone();
        if !name.is_empty() && name != self.totals.character {
            // different character: fresh totals
            self.finish_run();
            self.totals = Totals { character: name, ..Default::default() };
            self.save();
        }
        if self.run.as_ref().map_or(true, |r| r.seed != gd.seed_values.map_seed) {
            self.finish_run();
            self.run = Some(Run {
                seed: gd.seed_values.map_seed,
                started: Instant::now(),
                start_xp: None,
                xp_now: None,
                dead: HashSet::new(),
            });
        }
        let run = self.run.as_mut().unwrap();
        if let Some(xp) = experience(gd) {
            run.start_xp.get_or_insert(xp);
            run.xp_now = Some(xp);
        }
        for npc in gd.npcs.iter() {
            if matches!(npc.npc_type, NPCType::Monster | NPCType::Boss) && matches!(npc.mode, NPCMode::Dead | NPCMode::Death) {
                run.dead.insert(npc.unit_id);
            }
        }
    }

    fn run_xp(run: &Run) -> u64 {
        match (run.start_xp, run.xp_now) {
            (Some(a), Some(b)) if b >= a => (b - a) as u64,
            _ => 0,
        }
    }

    fn finish_run(&mut self) {
        let Some(run) = self.run.take() else { return };
        let secs = run.started.elapsed().as_secs_f64();
        if secs < MIN_RUN_SECS {
            return;
        }
        let t = &mut self.totals;
        t.runs += 1;
        t.total_secs += secs;
        t.last_secs = Some(secs);
        t.best_secs = Some(t.best_secs.map_or(secs, |b| b.min(secs)));
        t.kills += run.dead.len() as u64;
        t.xp += Self::run_xp(&run);
        self.save();
    }

    pub fn draw(&self, draw: &mut Draw, settings: &Settings, fonts: &Fonts, width: f32, height: f32) {
        if !settings.run_stats.enabled {
            return;
        }
        let Some(run) = self.run.as_ref() else { return };
        let t = &self.totals;
        let now = run.started.elapsed().as_secs_f64();
        let run_xp = Self::run_xp(run);
        let kills = run.dead.len() as u64;
        let total_secs = t.total_secs + now;
        let xp_hr = if total_secs > 30.0 { (t.xp + run_xp) as f64 * 3600.0 / total_secs } else { 0.0 };
        let avg = if t.runs > 0 { Some(t.total_secs / t.runs as f64) } else { None };
        let dash = |o: Option<f64>| o.map(fmt_time).unwrap_or_else(|| "-".into());

        let lines = [
            format!("Run {}   {}", t.runs + 1, fmt_time(now)),
            format!("Last {}   Avg {}   Best {}", dash(t.last_secs), dash(avg), dash(t.best_secs)),
            format!("Kills {}   ({} total)", kills, t.kills + kills),
            format!("XP {}   {}/hr", fmt_num(run_xp as f64), fmt_num(xp_hr)),
        ];
        let font = fonts.get_safe_font(&settings.general.language);
        let size = settings.run_stats.text_size;
        let x = width * settings.run_stats.pos_x;
        let mut y = height * settings.run_stats.pos_y;
        for line in lines.iter() {
            draw.text(&font, line).position(x + 1.5, y + 1.5).size(size).color(Color::BLACK).h_align_right().v_align_top();
            draw.text(&font, line)
                .position(x, y)
                .size(size)
                .color(Color::from_rgb(0.78, 0.70, 0.46))
                .h_align_right()
                .v_align_top();
            y += size * 1.3;
        }
    }
}
