//! Reimagined Data: a small always-on-top window with the d2r-reimagined.com item data
//! (uniques, sets, runewords, bases, affixes, cube recipes, orbs) and website-style tooltips.
//! It snaps to the top-left corner of the game window. Ctrl+Alt+F folds it away.

#![windows_subsystem = "windows"]

mod data;
mod format;

use std::path::PathBuf;
use std::time::{Duration, Instant};

use eframe::egui::{self, Color32, FontFamily, RichText};

use data::{Entry, Kind, Tab};

const EXOCET: &[u8] = include_bytes!("../../../src/gui/fonts/exocet.otf");
const FULL_SIZE: [f32; 2] = [720.0, 620.0];
const FOLDED_SIZE: [f32; 2] = [230.0, 34.0];

fn colour(k: Kind) -> Color32 {
    match k {
        Kind::Unique => Color32::from_rgb(0xc7, 0xb7, 0x90),
        Kind::Set => Color32::from_rgb(0x4c, 0xc2, 0x38),
        Kind::Rarity => Color32::from_rgb(0x63, 0xc7, 0xef),
        Kind::Base => Color32::from_rgb(0xb5, 0xb5, 0xb5),
        Kind::Requirement => Color32::from_rgb(0xe9, 0x6b, 0x63),
        Kind::Magic => Color32::from_rgb(0x63, 0x77, 0xef),
        Kind::Plain => Color32::from_rgb(0xdd, 0xd6, 0xc4),
        Kind::Heading => Color32::from_rgb(0xc7, 0xb7, 0x90),
        Kind::Note => Color32::from_rgb(0x9a, 0x93, 0x80),
    }
}

fn exocet() -> FontFamily {
    FontFamily::Name("exocet".into())
}

struct App {
    entries: Vec<Entry>,
    error: Option<String>,
    tab: Tab,
    query: String,
    results: Vec<usize>,
    dirty: bool,
    selected: Option<usize>,
    pinned: Vec<usize>,
    folded: bool,
    snap: bool,
    last_snap: Instant,
    hotkey_down: bool,
}

fn data_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("reimagined-data")
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert("exocet".to_owned(), std::sync::Arc::new(egui::FontData::from_static(EXOCET)));
        // Exocet first, normal fonts after it for characters Exocet does not have
        let mut fam = vec!["exocet".to_owned()];
        fam.extend(fonts.families.get(&FontFamily::Proportional).cloned().unwrap_or_default());
        fonts.families.insert(exocet(), fam);
        cc.egui_ctx.set_fonts(fonts);
        cc.egui_ctx.set_visuals(egui::Visuals::dark());

        let (entries, error) = match data::load(&data_dir()) {
            Ok((_, e)) => (e, None),
            Err(e) => (vec![], Some(e)),
        };
        App {
            entries,
            error,
            tab: Tab::Uniques,
            query: String::new(),
            results: vec![],
            dirty: true,
            selected: None,
            pinned: vec![],
            folded: false,
            snap: true,
            last_snap: Instant::now() - Duration::from_secs(5),
            hotkey_down: false,
        }
    }

    fn refilter(&mut self) {
        let words: Vec<String> = self.query.to_lowercase().split_whitespace().map(|w| w.to_string()).collect();
        let tab = self.tab;
        self.results = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| e.tab == tab && words.iter().all(|w| e.search.contains(w.as_str())))
            .map(|(i, _)| i)
            .collect();
        self.results.sort_by(|a, b| self.entries[*a].title.cmp(&self.entries[*b].title));
        self.dirty = false;
    }

    fn set_folded(&mut self, ctx: &egui::Context, folded: bool) {
        self.folded = folded;
        let size = if folded { FOLDED_SIZE } else { FULL_SIZE };
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(size[0], size[1])));
        self.last_snap = Instant::now() - Duration::from_secs(5);
    }

    /// Keep the window in the top-left corner of the game's window.
    fn snap_to_game(&mut self, ctx: &egui::Context) {
        if !self.snap || self.last_snap.elapsed() < Duration::from_millis(500) {
            return;
        }
        self.last_snap = Instant::now();
        if let Some((x, y)) = game_top_left() {
            let ppp = ctx.pixels_per_point().max(0.5);
            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2((x + 8) as f32 / ppp, (y + 8) as f32 / ppp)));
        }
    }

    fn hotkey(&mut self, ctx: &egui::Context) {
        let down = hotkey_pressed();
        if down && !self.hotkey_down {
            let f = !self.folded;
            self.set_folded(ctx, f);
        }
        self.hotkey_down = down;
    }
}

fn badge(ui: &mut egui::Ui, text: &str, red: bool) {
    let (fill, fg) = if red {
        (Color32::from_rgb(0x7a, 0x1a, 0x1a), Color32::from_rgb(0xf0, 0xb0, 0xa8))
    } else {
        (Color32::from_rgb(0x12, 0x1c, 0x2c), Color32::from_rgb(0xb8, 0xc4, 0xd8))
    };
    egui::Frame::default().fill(fill).inner_margin(egui::Margin::symmetric(6, 2)).show(ui, |ui| {
        ui.label(RichText::new(text).family(exocet()).size(11.0).color(fg));
    });
}

/// Draw one entry like the website's item card.
fn tooltip(ui: &mut egui::Ui, e: &Entry) {
    ui.vertical_centered(|ui| {
        ui.label(RichText::new(&e.title).family(exocet()).size(19.0).color(colour(e.title_kind)));
        if !e.subtitle.is_empty() {
            ui.label(RichText::new(&e.subtitle).family(exocet()).size(14.0).color(colour(Kind::Rarity)));
        }
        if !e.badges.is_empty() {
            ui.horizontal_wrapped(|ui| {
                // centre the row of badges
                let w: f32 = e.badges.iter().map(|b| b.text.len() as f32 * 7.5 + 16.0).sum();
                ui.add_space(((ui.available_width() - w) / 2.0).max(0.0));
                for b in &e.badges {
                    badge(ui, &b.text, b.red);
                }
            });
        }
        ui.add_space(4.0);
        ui.separator();
        ui.add_space(2.0);
        for l in &e.lines {
            let size = if l.kind == Kind::Heading { 16.0 } else { 14.0 };
            let text = if l.indent { format!("• {}", l.text) } else { l.text.clone() };
            if l.kind == Kind::Heading {
                ui.add_space(6.0);
            }
            ui.label(RichText::new(text).family(exocet()).size(size).color(colour(l.kind)));
        }
    });
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(150));
        self.hotkey(ctx);
        self.snap_to_game(ctx);

        if self.folded {
            egui::CentralPanel::default().show(ctx, |ui| {
                if ui.button(RichText::new("Reimagined Data  (Ctrl+Alt+F)").family(exocet()).size(12.0)).clicked() {
                    self.set_folded(ctx, false);
                }
            });
            return;
        }

        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Reimagined Data").family(exocet()).size(16.0).color(colour(Kind::Unique)));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Fold (Ctrl+Alt+F)").clicked() {
                        self.set_folded(ctx, true);
                    }
                    ui.checkbox(&mut self.snap, "Snap to game");
                });
            });
            ui.horizontal(|ui| {
                let r = ui.add(egui::TextEdit::singleline(&mut self.query).hint_text("Search name or stat…").desired_width(f32::INFINITY));
                if r.changed() {
                    self.dirty = true;
                }
            });
            ui.horizontal_wrapped(|ui| {
                for t in Tab::ALL {
                    if ui.selectable_label(self.tab == t, RichText::new(t.label()).family(exocet()).size(13.0)).clicked() && self.tab != t {
                        self.tab = t;
                        self.dirty = true;
                        self.selected = None;
                    }
                }
            });
            ui.add_space(2.0);
        });

        if self.dirty {
            self.refilter();
        }

        egui::SidePanel::left("list").resizable(true).default_width(250.0).show(ctx, |ui| {
            if let Some(err) = &self.error {
                ui.colored_label(colour(Kind::Requirement), format!("No data found.\n{}\n\nRun build_primemh_libd2.bat to download it.", err));
                return;
            }
            ui.label(RichText::new(format!("{} results", self.results.len())).size(11.0).color(colour(Kind::Note)));
            let row_h = 20.0;
            let results = self.results.clone();
            egui::ScrollArea::vertical().auto_shrink([false, false]).show_rows(ui, row_h, results.len(), |ui, range| {
                for i in range {
                    let idx = results[i];
                    let e = &self.entries[idx];
                    let text = RichText::new(&e.title).family(exocet()).size(13.0).color(colour(e.title_kind));
                    if ui.selectable_label(self.selected == Some(idx), text).clicked() {
                        self.selected = Some(idx);
                    }
                }
            });
        });

        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(Color32::from_rgb(0x14, 0x14, 0x14)).inner_margin(10.0))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    match self.selected {
                        Some(idx) => {
                            ui.horizontal(|ui| {
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    let pinned = self.pinned.contains(&idx);
                                    if ui.button(if pinned { "Unpin" } else { "Pin" }).clicked() {
                                        if pinned {
                                            self.pinned.retain(|p| *p != idx);
                                        } else {
                                            self.pinned.push(idx);
                                        }
                                    }
                                });
                            });
                            tooltip(ui, &self.entries[idx]);
                        }
                        None => {
                            ui.vertical_centered(|ui| {
                                ui.add_space(40.0);
                                ui.label(RichText::new("Pick an item on the left").family(exocet()).size(15.0).color(colour(Kind::Note)));
                            });
                        }
                    }
                    let others: Vec<usize> = self.pinned.iter().copied().filter(|p| Some(*p) != self.selected).collect();
                    if !others.is_empty() {
                        ui.add_space(12.0);
                        ui.separator();
                        ui.label(RichText::new("Pinned").family(exocet()).size(13.0).color(colour(Kind::Note)));
                        for idx in others {
                            ui.add_space(6.0);
                            egui::Frame::default().fill(Color32::from_rgb(0x1c, 0x1c, 0x1c)).inner_margin(8.0).show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.small_button("✕").clicked() {
                                            self.pinned.retain(|p| *p != idx);
                                        }
                                    });
                                });
                                tooltip(ui, &self.entries[idx]);
                            });
                        }
                    }
                });
            });
    }
}

// ---- Windows helpers -------------------------------------------------------------------

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Screen position (physical pixels) of the game window's client area, if the game is running.
fn game_top_left() -> Option<(i32, i32)> {
    use winapi::shared::windef::{POINT, RECT};
    use winapi::um::winuser::{ClientToScreen, FindWindowW, GetClientRect, IsIconic};
    unsafe {
        let title = wide("Diablo II: Resurrected");
        let hwnd = FindWindowW(std::ptr::null(), title.as_ptr());
        if hwnd.is_null() || IsIconic(hwnd) != 0 {
            return None;
        }
        let mut rect: RECT = std::mem::zeroed();
        if GetClientRect(hwnd, &mut rect) == 0 {
            return None;
        }
        let mut p = POINT { x: rect.left, y: rect.top };
        if ClientToScreen(hwnd, &mut p) == 0 {
            return None;
        }
        Some((p.x, p.y))
    }
}

fn hotkey_pressed() -> bool {
    use winapi::um::winuser::{GetAsyncKeyState, VK_CONTROL, VK_MENU};
    unsafe {
        let down = |k: i32| (GetAsyncKeyState(k) as u16 & 0x8000) != 0;
        down(VK_CONTROL) && down(VK_MENU) && down('F' as i32)
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Reimagined Data")
            .with_inner_size(FULL_SIZE)
            .with_always_on_top(),
        ..Default::default()
    };
    eframe::run_native("Reimagined Data", options, Box::new(|cc| Ok(Box::new(App::new(cc)))))
}
