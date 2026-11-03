use std::collections::HashMap;

use notan::draw::*;
use notan::prelude::*;

use crate::gui::Fonts;
use crate::memory::gamedata::GameData;
use crate::settings::Settings;
use crate::types::object::GameObjectMode;
use crate::types::object::GameObjectType;
use crate::types::object::GameObjectUnit;

pub fn draw_objects(draw: &mut Draw, game_data: &GameData, settings: &Settings, width: &f32, height: &f32, images: &HashMap<String, Texture>, _font: &Fonts) {
    let player_pos = (game_data.player.pos_x, game_data.player.pos_y);

    let chest_image = images.get("chest").unwrap();
    let super_chest_image = images.get("superchest").unwrap();

    game_data.objects.iter().for_each(|object| match object.object_type {
        GameObjectType::Chest => draw_chest(object, player_pos, draw, settings, chest_image, width, height),
        GameObjectType::Portal => draw_portal(object, player_pos, draw, settings, width, height),
        GameObjectType::RedPortal => draw_red_portal(object, player_pos, draw, settings, width, height),
        GameObjectType::SuperChest => draw_super_chest(object, player_pos, draw, settings, super_chest_image, width, height),
        GameObjectType::Shrine => (),
        GameObjectType::Well => (),
        GameObjectType::ArmorRack => (),
        GameObjectType::Dummy => (),
    });
}

fn draw_chest(
    chest: &GameObjectUnit,
    player_pos: (f32, f32),
    draw: &mut Draw,
    settings: &Settings,
    chest_image: &Texture,
    width: &f32, 
    height: &f32
) {
    if chest.chest_state.is_none() || chest.mode != GameObjectMode::Neutral || !settings.chests.enabled || settings.chests.super_only {
        return;
    }
    let scale = settings.visual.scale;
    let unit_pos = (chest.pos_x, chest.pos_y);
    let h = (chest_image.height() * settings.chests.size) * scale;
    let w = (chest_image.width() * settings.chests.size) * scale;
    let chest_pos = transform_position(unit_pos, (h, w), player_pos, scale, width, height);

    if let Some(state) = &chest.chest_state {
        let color = if state.trapped {
            Color::RED
        } else if state.locked {
            Color::AQUA
        } else {
            Color::WHITE
        };
        draw.image(chest_image)
            .size(w, h)
            .position(chest_pos.0, chest_pos.1)
            .color(color);
    }
}

fn draw_super_chest(
    chest: &GameObjectUnit,
    player_pos: (f32, f32),
    draw: &mut Draw,
    settings: &Settings,
    super_chest_image: &Texture,
    width: &f32, 
    height: &f32
) {
    if chest.chest_state.is_none() || chest.mode != GameObjectMode::Neutral || !settings.chests.enabled {
        // neutral means opened
        return;
    }
    let scale = settings.visual.scale;
    let unit_pos = (chest.pos_x, chest.pos_y);
    let h = (super_chest_image.height() * settings.chests.size) * scale;
    let w = (super_chest_image.width() * settings.chests.size) * scale;
    let chest_pos = transform_position(unit_pos, (h, w), player_pos, scale, width, height);
    draw.image(super_chest_image)
        .size(w, h)
        .position(chest_pos.0, chest_pos.1);
}


fn draw_portal(portal: &GameObjectUnit, player_pos: (f32, f32), draw: &mut Draw, settings: &Settings, width: &f32, height: &f32) {
    if !settings.portals.enabled {
        return;
    }
    let scale = settings.visual.scale;
    let unit_pos = (portal.pos_x, portal.pos_y);
    let portal_size = (settings.portals.size * scale, (settings.portals.size * 1.8) * scale);
    let portal_pos = transform_position(unit_pos, portal_size, player_pos, scale, width, height);
    
    draw.ellipse(portal_pos, portal_size)
        .stroke(1.0 * scale)
        .color(Color::from_hex(0x00AAFFFF));
}

fn draw_red_portal(portal: &GameObjectUnit, player_pos: (f32, f32), draw: &mut Draw, settings: &Settings, width: &f32, height: &f32) {
    if !settings.portals.enabled {
        return;
    }
    let scale = settings.visual.scale;
    let unit_pos = (portal.pos_x, portal.pos_y);
    let portal_size = (settings.portals.size * scale, (settings.portals.size * 1.8) * scale);
    let portal_pos = transform_position(unit_pos, portal_size, player_pos, scale, width, height);
    draw.ellipse(portal_pos, portal_size)
        .stroke(1.0 * scale)
        .color(Color::RED);
}

fn transform_position(
    unit_pos: (u32, u32),
    size: (f32, f32),
    player_pos: (f32, f32),
    scale: f32,
    width: &f32, 
    height: &f32
) -> (f32, f32) {
    let xdiff = unit_pos.0 as f32 - player_pos.0;
    let ydiff = unit_pos.1 as f32 - player_pos.1;

    let center_x = *width as f32 / 2.0;
    let center_y = *height as f32 / 2.0;
    let angle: f32 = std::f32::consts::FRAC_PI_4;
    let x = xdiff * angle.cos() - ydiff * angle.sin();
    let y = xdiff * angle.sin() + ydiff * angle.cos();

    let new_pos_x = center_x + (x * scale) - (size.0 / 2.0);
    let new_pos_y = center_y + (y * scale * 0.5) - (size.1 / 2.0);

    (new_pos_x, new_pos_y)
}
