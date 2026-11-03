use notan::egui::{self, *};
use notan::prelude::*;
use winapi::shared::windef::HWND;
use crate::settings::{Locales, MapPosition};
use crate::LOCALISATION;
use super::ui::State;


pub fn create_egui_panel(app: &mut App, ctx: &Context, state: &mut State, hwnd: HWND) {
    setup_custom_fonts(ctx);
    let mut localisation = LOCALISATION.lock().unwrap();
    
    if localisation.current_locale != state.settings.general.language {
        localisation.update_locale(state.settings.general.language.clone());
    }
    
    ctx.set_pixels_per_point(app.window().dpi() as f32);
    egui::Window::new(egui::RichText::new("PrimeMH").size(15.0)).default_open(false).show(ctx, |ui| {
        egui::Grid::new("helper_text")
        .num_columns(2)
        .spacing([6.0, 6.0])
        .striped(true)
        .show(ui, |ui| {
            ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                let toggle_text = format!(
                    "{} {}\n{} {}",
                    localisation.get_primemh("hide_ui"),
                    state.settings.hotkeys.hotkey_toggle_menu,
                    localisation.get_primemh("hide_map"),
                    state.settings.hotkeys.hotkey_toggle_map,
                );
                ui.label(toggle_text);
                
            });
            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                
                if ui
                    .add(egui::Button::image(egui::Image::new(state.language_icon)
                        .max_width(25.0)
                        .tint(egui::Color32::GRAY)))
                    .clicked()
                {
                    state.language_selector_visible = true;
                }
            });
        });
        
        ui.separator();
        
        egui::CollapsingHeader::new(localisation.get_primemh("map_settings"))
            .default_open(false)
            .show(ui, |ui| {
                egui::Grid::new("settings_grid")
                    .num_columns(2)
                    .spacing([20.0, 6.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label(localisation.get_primemh("map_scale"));
                        ui.add(egui::Slider::new(&mut state.settings.visual.scale, 0.1..=8.0));
                        ui.end_row();

                        ui.label(localisation.get_primemh("map_opacity"));
                        ui.add(egui::Slider::new(&mut state.settings.visual.map_opacity, 0.0..=1.0));
                        
                        ui.end_row();

                        ui.label(localisation.get_primemh("always_show_map"));
                        ui.add(egui::Checkbox::new(&mut state.settings.visual.always_show_map, ""));
                        ui.end_row();

                        ui.label(localisation.get_primemh("hide_map_in_menus"));
                        ui.add(egui::Checkbox::new(&mut state.settings.visual.hide_map_menus_open, ""));
                        ui.end_row();

                        ui.label(localisation.get_primemh("map_position"));
                        ui.horizontal(|ui| {
                            ui.radio_value(
                                &mut state.settings.general.map_position,
                                MapPosition::Center,
                                localisation.get_primemh("center"),
                            );
                            ui.radio_value(
                                &mut state.settings.general.map_position,
                                MapPosition::TopLeft,
                                localisation.get_primemh("top_left"),
                            );
                            ui.radio_value(
                                &mut state.settings.general.map_position,
                                MapPosition::TopRight,
                                localisation.get_primemh("top_right"),
                            );
                        });
                        ui.end_row();
                        
                        
                        ui.label(localisation.get_primemh("exit_text_size"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.visual.exit_label_text_size)
                                .range(0.1..=15.0)
                                .speed(0.1),
                        );
                        ui.end_row();
                        
                        
                    });
            });

        ui.separator();
        egui::CollapsingHeader::new(localisation.get_primemh("map_objects"))
            .default_open(false)
            .show(ui, |ui| {
                egui::Grid::new("objects_grid")
                    .num_columns(2)
                    .spacing([6.0, 6.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label(localisation.get_primemh("show_chests"));
                        ui.add(egui::Checkbox::new(&mut state.settings.chests.enabled, ""));
                        ui.label("Super chests only");
                        ui.add(egui::Checkbox::new(&mut state.settings.chests.super_only, ""));
                        ui.label(localisation.get_primemh("size"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.chests.size)
                                .range(0.01..=1.0)
                                .speed(0.01),
                        );
                        ui.end_row();

                        ui.label(localisation.get_primemh("show_portals"));
                        ui.add(egui::Checkbox::new(&mut state.settings.portals.enabled, ""));
                        ui.label(localisation.get_primemh("size"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.portals.size)
                                .range(0.1..=8.0)
                                .speed(0.1),
                        );
                        ui.label(localisation.get_primemh("show_portal_area_name"));
                        ui.add(egui::Checkbox::new(&mut state.settings.portals.show_area_name, ""));
                        ui.label(localisation.get_primemh("text_size"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.portals.portal_font_size)
                                .range(1.0..=20.0)
                                .speed(0.5),
                        );
                        ui.end_row();

                        ui.label(localisation.get_primemh("show_shrines"));
                        ui.add(egui::Checkbox::new(&mut state.settings.shrines.enabled, ""));
                        ui.label(localisation.get_primemh("size"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.shrines.size)
                                .range(0.01..=1.0)
                                .speed(0.01),
                        );
                        ui.label(localisation.get_primemh("text_size"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.shrines.text_size)
                                .range(1.0..=20.0)
                                .speed(0.5),
                        );
                        ui.end_row();
                    });
                egui::CollapsingHeader::new("Shrine types to show").default_open(false).show(ui, |ui| {
                    egui::Grid::new("shrine_types_grid").num_columns(3).spacing([12.0, 4.0]).show(ui, |ui| {
                        const TYPES: [&str; 23] = [
                            "Experience", "Skill", "Gem", "Combat", "Armor", "ManaRecharge", "Stamina",
                            "ResistFire", "ResistCold", "ResistLightning", "ResistPoison",
                            "Refilling", "Health", "Mana", "HealthExchange", "ManaExchange",
                            "Enirhs", "Portal", "Fire", "Monster", "Exploding", "Poison", "Unknown",
                        ];
                        for (i, t) in TYPES.iter().enumerate() {
                            let mut shown = !state.settings.shrines.hidden.iter().any(|h| h == t);
                            let label = if *t == "Unknown" { "Not identified yet" } else { t };
                            if ui.checkbox(&mut shown, label).changed() {
                                state.settings.shrines.hidden.retain(|h| h != t);
                                if !shown {
                                    state.settings.shrines.hidden.push(t.to_string());
                                }
                            }
                            if i % 3 == 2 {
                                ui.end_row();
                            }
                        }
                    });
                });
            });

        ui.separator();
        egui::CollapsingHeader::new(localisation.get_primemh("monsters"))
            .default_open(false)
            .show(ui, |ui| {
                egui::Grid::new("monsters_grid")
                    .num_columns(2)
                    .spacing([20.0, 6.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label(localisation.get_primemh("show_monsters"));
                        ui.add(egui::Checkbox::new(&mut state.settings.monsters.enabled, ""));
                        ui.end_row();

                        ui.label(localisation.get_primemh("show_immunities"));
                        ui.add(egui::Checkbox::new(&mut state.settings.monsters.immunities, ""));
                        ui.end_row();

                        ui.label(localisation.get_primemh("normal_mobs"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.monsters.normal_mobs_size)
                                .range(0.5..=10.0)
                                .speed(0.1),
                        );
                        ui.color_edit_button_srgba_unmultiplied(&mut state.settings.monsters.normal_mob_color);
                        ui.end_row();

                        ui.label(localisation.get_primemh("minions"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.monsters.minions_mobs_size)
                                .range(0.5..=10.0)
                                .speed(0.1),
                        );
                        ui.color_edit_button_srgba_unmultiplied(&mut state.settings.monsters.minions_mob_color);
                        ui.end_row();

                        ui.label(localisation.get_primemh("champions"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.monsters.champions_mobs_size)
                                .range(0.5..=10.0)
                                .speed(0.1),
                        );
                        ui.color_edit_button_srgba_unmultiplied(&mut state.settings.monsters.champions_mob_color);
                        ui.end_row();

                        ui.label(localisation.get_primemh("uniques"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.monsters.unique_mobs_size)
                                .range(0.5..=10.0)
                                .speed(0.1),
                        );
                        ui.color_edit_button_srgba_unmultiplied(&mut state.settings.monsters.unique_mob_color);
                        ui.end_row();

                        ui.label(localisation.get_primemh("bosses"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.monsters.boss_mobs_size)
                                .range(0.5..=10.0)
                                .speed(0.1),
                        );
                        ui.color_edit_button_srgba_unmultiplied(&mut state.settings.monsters.boss_mob_color);
                        ui.end_row();
                    });
            });

        ui.separator();
        egui::CollapsingHeader::new(localisation.get_primemh("lines_and_pathfinding"))
            .default_open(false)
            .show(ui, |ui| {
                egui::Grid::new("lines_grid")
                    .num_columns(2)
                    .spacing([20.0, 6.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label(localisation.get_primemh("show_waypoint_lines"));
                        ui.add(egui::Checkbox::new(&mut state.settings.lines.waypoint_enabled, "Line"));
                        ui.add(egui::Checkbox::new(&mut state.settings.lines.waypoint_path_enabled, "Path"));
                        ui.color_edit_button_srgba_unmultiplied(&mut state.settings.lines.waypoint_rgba);
                        ui.end_row();

                        ui.label(localisation.get_primemh("show_exit_lines"));
                        ui.add(egui::Checkbox::new(&mut state.settings.lines.exit_enabled, "Line"));
                        ui.add(egui::Checkbox::new(&mut state.settings.lines.exit_path_enabled, "Path"));
                        ui.color_edit_button_srgba_unmultiplied(&mut state.settings.lines.exit_rgba);
                        ui.end_row();

                        ui.label(localisation.get_primemh("show_quest_lines"));
                        ui.add(egui::Checkbox::new(&mut state.settings.lines.quest_enabled, "Line"));
                        ui.add(egui::Checkbox::new(&mut state.settings.lines.quest_path_enabled, "Path"));
                        ui.color_edit_button_srgba_unmultiplied(&mut state.settings.lines.quest_rgba);
                        ui.end_row();

                        ui.label(localisation.get_primemh("show_boss_lines"));
                        ui.add(egui::Checkbox::new(&mut state.settings.lines.boss_enabled, "Line"));
                        ui.add(egui::Checkbox::new(&mut state.settings.lines.boss_path_enabled, "Path"));
                        ui.color_edit_button_srgba_unmultiplied(&mut state.settings.lines.boss_rgba);
                        ui.end_row();

                        ui.label(localisation.get_primemh("line_width"));
                        ui.add(
                        egui::DragValue::new(&mut state.settings.lines.line_width)
                            .range(0.5..=10.0)
                            .speed(0.1),
                        );
                        ui.end_row();
                    });
            });
        ui.separator();
        egui::CollapsingHeader::new(localisation.get_primemh("buff_bar")).default_open(false).show(ui, |ui| {
            egui::Grid::new("buffbar_grid")
                .num_columns(2)
                .spacing([20.0, 6.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label(localisation.get_primemh("buff_bar_show"));
                    ui.add(egui::Checkbox::new(&mut state.settings.buffbar.enabled, ""));
                    ui.end_row();
                    ui.label(localisation.get_primemh("buff_bar_icon_size"));
                    ui.add(
                        egui::DragValue::new(&mut state.settings.buffbar.icon_scale)
                            .range(1.0..=50.0)
                            .speed(0.25),
                    );
                    ui.end_row();
                    ui.label(localisation.get_primemh("buff_bar_vert_pos"));
                    ui.add(
                        egui::DragValue::new(&mut state.settings.buffbar.vertical_pos)
                            .range(0.0..=1.0)
                            .speed(0.001),
                    );
                    ui.end_row();
                    ui.label(localisation.get_primemh("buff_bar_horiz_pos"));
                    ui.add(
                        egui::DragValue::new(&mut state.settings.buffbar.horizontal_pos)
                            .range(0.0..=1.0)
                            .speed(0.001),
                    );
                    ui.end_row();
                    ui.label(localisation.get_primemh("charges_font_size"));
                    ui.add(
                        egui::DragValue::new(&mut state.settings.buffbar.charges_font_size)
                            .range(1.0..=40.0)
                            .speed(0.5),
                    );
                    ui.end_row();
                    ui.label(localisation.get_primemh("timer_font_size"));
                    ui.add(
                        egui::DragValue::new(&mut state.settings.buffbar.timer_font_size)
                            .range(1.0..=40.0)
                            .speed(0.5),
                    );
                    ui.end_row();
                    
                });
        });
        ui.separator();
        egui::CollapsingHeader::new(localisation.get_primemh("party_info")).default_open(false).show(ui, |ui| {
            egui::Grid::new("party_grid")
                .num_columns(2)
                .spacing([20.0, 6.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label(localisation.get_primemh("party_info_enabled"));
                    ui.add(egui::Checkbox::new(&mut state.settings.party_info.enabled, ""));
                    ui.end_row();
                    ui.label("Drop log (uniques and sets)");
                    ui.add(egui::Checkbox::new(&mut state.settings.drop_log.enabled, ""));
                    ui.end_row();
                    ui.label("Drop log seconds shown");
                    ui.add(egui::DragValue::new(&mut state.settings.drop_log.duration_secs).range(5..=3600).speed(5));
                    ui.end_row();
                    ui.label("Drop log text size");
                    ui.add(egui::DragValue::new(&mut state.settings.drop_log.text_size).range(6.0..=40.0).speed(0.5));
                    ui.end_row();
                    ui.label("Run timer and stats");
                    ui.add(egui::Checkbox::new(&mut state.settings.run_stats.enabled, ""));
                    ui.end_row();
                    ui.label("Run stats text size");
                    ui.add(egui::DragValue::new(&mut state.settings.run_stats.text_size).range(6.0..=40.0).speed(0.5));
                    ui.end_row();
                    if ui.button("Reset run stats").clicked() {
                        state.run_stats.reset();
                    }
                    ui.end_row();
                    ui.label("Friend tracking (line / arrow)");
                    ui.add(egui::Checkbox::new(&mut state.settings.party_info.friend_tracking, ""));
                    ui.end_row();
                    
                });
        });
        ui.separator();
        egui::CollapsingHeader::new("Drop filter and sound").default_open(false).show(ui, |ui| {
            let dl = &mut state.settings.drop_log;
            ui.checkbox(&mut dl.only_watched, "Only list ticked items in the drop log");
            ui.checkbox(&mut dl.sound, "Play a sound when a ticked item drops");
            ui.horizontal(|ui| {
                ui.label("Volume");
                ui.add(egui::Slider::new(&mut dl.volume, 0.0..=1.5));
                if ui.button("Test sound").clicked() {
                    crate::droplog::test_sound(&dl.sound_file, dl.volume);
                }
            });
            ui.horizontal(|ui| {
                ui.label("Sound file");
                ui.add(egui::TextEdit::singleline(&mut dl.sound_file).desired_width(140.0));
            });
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Search");
                ui.add(egui::TextEdit::singleline(&mut state.drop_log.search).hint_text("item name or base").desired_width(160.0));
                if ui.button("x").clicked() {
                    state.drop_log.search.clear();
                }
            });
            ui.horizontal(|ui| {
                ui.label(format!("{} ticked", dl.watch.len()));
                if !dl.watch.is_empty() && ui.button("Untick all").clicked() {
                    dl.watch.clear();
                }
            });
            if state.drop_log.catalog.is_empty() {
                ui.label("Item list not found - run the build script so it downloads reimagined-data.");
            }
            let query = state.drop_log.search.trim().to_lowercase();
            egui::ScrollArea::vertical().max_height(260.0).auto_shrink([false, true]).show(ui, |ui| {
                // ticked items first, then everything matching the search
                for pass in 0..2 {
                    for item in state.drop_log.catalog.iter() {
                        let ticked = dl.watch.iter().any(|w| *w == item.name);
                        if (pass == 0) != ticked {
                            continue;
                        }
                        if !query.is_empty() && !item.search.contains(&query) {
                            continue;
                        }
                        let colour = if item.set { egui::Color32::from_rgb(77, 194, 56) } else { egui::Color32::from_rgb(199, 179, 119) };
                        let label = if item.base.is_empty() { item.name.clone() } else { format!("{}  ({})", item.name, item.base) };
                        let mut on = ticked;
                        if ui.checkbox(&mut on, egui::RichText::new(label).color(colour)).changed() {
                            if on {
                                dl.watch.push(item.name.clone());
                            } else {
                                dl.watch.retain(|w| *w != item.name);
                            }
                        }
                    }
                }
            });
            ui.label("Press Save at the bottom to keep your ticks.");
        });
        ui.separator();
        egui::CollapsingHeader::new(localisation.get_primemh("missiles"))
            .default_open(false)
            .show(ui, |ui| {
                egui::Grid::new("missiles_grid")
                    .num_columns(2)
                    .spacing([20.0, 6.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label(localisation.get_primemh("show_missiles"));
                        ui.add(egui::Checkbox::new(&mut state.settings.missiles.enabled, ""));
                        ui.end_row();

                        // Fire
                        ui.label(localisation.get_primemh("fire"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.missiles.fire_size)
                                .range(0.1..=10.0)
                                .speed(0.1),
                        );
                        ui.color_edit_button_srgba_unmultiplied(&mut state.settings.missiles.fire_color);
                        ui.end_row();

                        // Cold
                        ui.label(localisation.get_primemh("cold"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.missiles.cold_size)
                                .range(0.1..=10.0)
                                .speed(0.1),
                        );
                        ui.color_edit_button_srgba_unmultiplied(&mut state.settings.missiles.cold_color);
                        ui.end_row();

                        // Poison
                        ui.label(localisation.get_primemh("poison"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.missiles.poison_size)
                                .range(0.1..=10.0)
                                .speed(0.1),
                        );
                        ui.color_edit_button_srgba_unmultiplied(&mut state.settings.missiles.poison_color);
                        ui.end_row();

                        // Lightning
                        ui.label(localisation.get_primemh("lightning"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.missiles.lightning_size)
                                .range(0.1..=10.0)
                                .speed(0.1),
                        );
                        ui.color_edit_button_srgba_unmultiplied(&mut state.settings.missiles.lightning_color);
                        ui.end_row();

                        // Physical
                        ui.label(localisation.get_primemh("physical"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.missiles.physical_size)
                                .range(0.1..=10.0)
                                .speed(0.1),
                        );
                        ui.color_edit_button_srgba_unmultiplied(&mut state.settings.missiles.physical_color);
                        ui.end_row();

                        // Magic
                        ui.label(localisation.get_primemh("magic"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.missiles.magic_size)
                                .range(0.1..=10.0)
                                .speed(0.1),
                        );
                        ui.color_edit_button_srgba_unmultiplied(&mut state.settings.missiles.magic_color);
                        ui.end_row();
                    });
            });

        ui.separator();
        ui.hyperlink("https://discord.gg/swswCBXbp6");

        let splash_text = format!(
            "{}{}{}{}{}",
            obfstr::obfstr!("If you paid for this, you have been scammed\n"),
            obfstr::obfstr!("如果您為此付出了，您已經被騙了\n"),
            obfstr::obfstr!("당신이 이것을 지불했다면 당신은 사기를당했습니다\n"),
            obfstr::obfstr!("あなたがこれに対して支払った場合、あなたは詐欺されています\n"),
            obfstr::obfstr!("Если вы заплатили за это, вас обманули\n")
        );
        
        ui.label(splash_text.clone());
        let title = format!("FPS {}", app.timer.fps().round());
        ui.label(title);

        
        if state.instance_locked.is_some() {
            if ui.add(egui::Button::image_and_text(egui::Image::new(state.locked_icon)
                    .max_width(18.0)
                    .tint(egui::Color32::WHITE), "Locked")).clicked() {
                state.instance_locked = None;
            }
        } else {
            if ui.add(egui::Button::image_and_text(egui::Image::new(state.unlocked_icon)
                    .max_width(18.0)
                    .tint(egui::Color32::WHITE), "Unlocked")).clicked() {
                state.instance_locked = Some(hwnd);
            }
        }

        ui.separator();
        ui.horizontal(|ui| {
            if ui.button(localisation.get_primemh("quit")).clicked() {
                app.exit();
            }
            if ui.button(localisation.get_primemh("save_settings")).clicked() {
                state.settings.save();
            }
        });
        
    });
}


pub fn create_language_select_ui(app: &mut App, ctx: &Context, state: &mut State) {
    setup_custom_fonts(ctx);
    ctx.set_pixels_per_point(app.window().dpi() as f32);
    egui::Window::new("Choose your language").default_open(true).collapsible(false).resizable(false).fixed_size((280.0, 280.0)).show(ctx, |ui| {
        ui.vertical_centered_justified(|ui| {
            let font_size = 20.0;
            if ui.button(egui::RichText::new("English").size(font_size)).clicked() { change_language(state, Locales::enUS)} 
            if ui.button(egui::RichText::new("Deutsch").size(font_size)).clicked() { change_language(state, Locales::deDE)}
            if ui.button(egui::RichText::new("한국어").size(font_size)).clicked() { change_language(state, Locales::koKR)}
            if ui.button(egui::RichText::new("臺灣話").size(font_size)).clicked() { change_language(state, Locales::zhTW)}
            if ui.button(egui::RichText::new("español").size(font_size)).clicked() { change_language(state, Locales::esES)}
            if ui.button(egui::RichText::new("français").size(font_size)).clicked() { change_language(state, Locales::frFR)}
            if ui.button(egui::RichText::new("italiano").size(font_size)).clicked() { change_language(state, Locales::itIT)}
            if ui.button(egui::RichText::new("polski").size(font_size)).clicked() { change_language(state, Locales::plPL)}
        });
    });
}


fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "blizzard".to_owned(),
        egui::FontData::from_static(include_bytes!("./fonts/blizzardglobaltcunicode.ttf")).into(),
    );
    // Put my font first (highest priority) for proportional text:
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "blizzard".to_owned());

    // Put my font as last fallback for monospace:
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("blizzard".to_owned());
    ctx.set_fonts(fonts);
}

fn change_language(state: &mut State, locale: Locales) {
    log::debug!("Selected new language {:?}", locale);
    state.settings.save_locale(locale);
    state.language_selector_visible = false;   
}
