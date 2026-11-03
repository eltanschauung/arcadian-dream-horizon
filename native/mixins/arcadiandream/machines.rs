// SPDX-License-Identifier: EUPL-1.2-or-later
use super::arcadian::{definitions, number, set_custom, shot};
use super::{Game, native_hooks::NativeScreen};
use horizon_client::containers::{ModBlockEntity, ModContainer};
use horizon_client::items::ItemStack;
use horizon_core::geometry::Direction;
use horizon_world::nbt::Compound;
use serde_json::{Value, json};
const PEDESTALS: [[i32; 3]; 16] = [
    [0, 0, -4],
    [3, 0, -3],
    [4, 0, 0],
    [3, 0, 3],
    [0, 0, 4],
    [-3, 0, 3],
    [-4, 0, 0],
    [-3, 0, -3],
    [-2, 1, -5],
    [2, 1, -5],
    [5, 1, -2],
    [5, 1, 2],
    [2, 1, 5],
    [-2, 1, 5],
    [-5, 1, 2],
    [-5, 1, -2],
];
fn pedestal(block: &str) -> bool {
    block == "arcadiandream:onbashira" || block == "arcadiandream:ritual_shrine"
}
pub fn entity(block: &str, out: &mut Option<(String, Compound)>) {
    if pedestal(block)
        || block == "arcadiandream:danmaku_crafting_table"
        || block == "arcadiandream:mysterious_seal"
    {
        *out = Some((block.into(), Compound::new()));
    }
}
pub fn container(block: &str, out: &mut Option<ModContainer>) {
    let slots = if pedestal(block) {
        1
    } else if block == "arcadiandream:danmaku_crafting_table" {
        7
    } else {
        return;
    };
    *out = Some(ModContainer {
        slots,
        faces: std::array::from_fn(|_| if slots == 7 { Vec::new() } else { vec![0] }),
        inputs: (0..slots).filter(|i| slots != 7 || *i != 2).collect(),
        outputs: if slots == 7 { vec![2] } else { vec![0] },
        menu: if slots == 7 {
            Some("arcadiandream:danmaku_crafting_table".into())
        } else {
            None
        },
    });
}
pub fn screen(block: &str, out: &mut Option<NativeScreen>) {
    if block == "arcadiandream:danmaku_crafting_table" {
        *out = Some(NativeScreen {
            width: 176.0,
            height: 166.0,
            slots: vec![
                Some([8.0, 35.0]),
                Some([35.0, 35.0]),
                Some([149.0, 35.0]),
                Some([62.0, 17.0]),
                Some([62.0, 53.0]),
                Some([89.0, 17.0]),
                Some([116.0, 17.0]),
            ],
            inventory_x: 8.0,
            inventory_y: 84.0,
            own_labels: false,
            own_background: false,
        });
    }
}
fn ingredient(v: &Value) -> Value {
    if let Some(s) = v["item"].as_str() {
        json!(s)
    } else if let Some(s) = v["tag"].as_str() {
        json!(format!("#{s}"))
    } else {
        v.clone()
    }
}
fn take(slot: &mut Option<ItemStack>, count: u32) {
    if let Some(s) = slot {
        s.count = s.count.saturating_sub(count);
        if s.count == 0 {
            *slot = None
        }
    }
}
impl Game<'_> {
    pub(super) fn arcadian_machine_click(&mut self, pos: [i32; 3]) -> bool {
        if !self.native_on("arcadiandream") {
            return false;
        }
        let block = self.native_block(pos);
        if !pedestal(&block) {
            return false;
        }
        let dim = self.dim.id().to_string();
        let Some(slots) = self.native_container_slots(&dim, pos, Direction::Up) else {
            return true;
        };
        if let Some(stack) = slots.first().and_then(|s| s.stack.as_ref()) {
            let stack = stack.clone();
            let id = self.native_acting_id();
            if let Some(taken) = self.native_extract_item(&dim, pos, Direction::Up, 0, stack.count)
            {
                if let Some(rest) = self.native_give_stack(id, taken) {
                    self.native_insert_item(pos, Direction::Up, rest);
                }
            }
            self.native_block_entity_sync(pos);
            return true;
        }
        if block == "arcadiandream:ritual_shrine" {
            let mut positions = Vec::new();
            let mut stacks = Vec::new();
            for off in PEDESTALS {
                let p = std::array::from_fn(|i| pos[i] + off[i]);
                if self.native_block(p) != "arcadiandream:onbashira" {
                    continue;
                }
                if let Some(s) = self
                    .native_container_slots(&dim, p, Direction::Up)
                    .and_then(|s| s.first().and_then(|s| s.stack.clone()))
                {
                    positions.push(p);
                    stacks.push(s);
                }
            }
            let Some(recipes) = definitions()["rituals"].as_object() else {
                return true;
            };
            for recipe in recipes.values() {
                let Some(ingredients) = recipe["ingredients"].as_array() else {
                    continue;
                };
                if ingredients.len() != stacks.len() {
                    continue;
                }
                if let Some(d) = recipe["dimension"].as_str() {
                    if d != "all" && dim.strip_prefix("minecraft:").unwrap_or(&dim) != d {
                        continue;
                    }
                }
                // Backtracking instead of greedy matching: overlapping item tags must not reject a valid ritual.
                let Some(parsed) = ingredients
                    .iter()
                    .map(|i| self.native_recipe_ingredient(&ingredient(i)))
                    .collect::<Option<Vec<_>>>()
                else {
                    continue;
                };
                fn matches(
                    i: usize,
                    ing: &[horizon_client::crafting::Ingredient],
                    stacks: &[ItemStack],
                    used: &mut [u8],
                ) -> bool {
                    if i == ing.len() {
                        return true;
                    }
                    for j in 0..stacks.len() {
                        if used[j] == 0 && ing[i].test(&stacks[j]) {
                            used[j] = 1;
                            if matches(i + 1, ing, stacks, used) {
                                return true;
                            }
                            used[j] = 0;
                        }
                    }
                    false
                }
                if !matches(0, &parsed, &stacks, &mut vec![0; stacks.len()]) {
                    continue;
                }
                let mut result = recipe["result"].clone();
                if result["id"].is_null() {
                    result["id"] = result["item"].clone();
                }
                let Some(result) = self.native_recipe_result(&result) else {
                    continue;
                };
                if self
                    .native_insert_item(pos, Direction::Up, result)
                    .is_some()
                {
                    return true;
                }
                for (p, s) in positions.into_iter().zip(stacks) {
                    self.native_extract_item(&dim, p, Direction::Up, 0, s.count);
                    self.native_block_entity_sync(p);
                }
                self.native_block_entity_sync(pos);
                let player = self.native_acting_id();
                self.native_trigger_criterion(player, "arcadiandream:ritual_crafting", |_| true);
                return true;
            }
            return true;
        }
        if let Some(held) = self.held().cloned() {
            let one = ItemStack { count: 1, ..held };
            if self.native_insert_item(pos, Direction::Up, one).is_none() {
                self.consume_held();
                self.native_block_entity_sync(pos);
            }
        }
        true
    }
    pub(super) fn arcadian_machine_tick(&mut self, pos: [i32; 3], e: &mut ModBlockEntity) {
        if e.id == "arcadiandream:mysterious_seal" {
            if self.world.game_time.rem_euclid(40) != 0 {
                return;
            }
            let dimension = self.dim.id().to_owned();
            for player in self
                .native_server_players()
                .into_iter()
                .filter(|p| p.dimension == dimension && p.game_mode != 1 && p.game_mode != 3)
            {
                if (0..3).all(|i| (player.position[i] - pos[i] as f64).abs() <= 16.0) {
                    let id = super::arcadian::uuid(player.uuid);
                    self.arcadian_queue(format!(
                        "effect give {id} minecraft:mining_fatigue 10 2 true"
                    ));
                }
            }
            let dim = self.dim.id().to_owned();
            let mut fires = Vec::new();
            for x in -16..=16 {
                for y in -16..=16 {
                    for z in -16..=16 {
                        let at = [pos[0] + x, pos[1] + y, pos[2] + z];
                        if matches!(
                            self.native_block(at).as_str(),
                            "minecraft:fire" | "minecraft:soul_fire"
                        ) {
                            fires.push(at);
                        }
                    }
                }
            }
            for at in fires {
                if let Some(state) = self.pack.assets.registry.default_state("minecraft:air") {
                    let _ = self.native_set_block(&dim, at, state, 3);
                }
            }
            return;
        }
        if e.id != "arcadiandream:danmaku_crafting_table" {
            return;
        }
        e.items.resize(7, None);
        if e.items[2].is_some() {
            return;
        }
        if e.items[0].is_some() && e.items[1].is_none() {
            let core = e.items[0].as_ref().unwrap();
            if let Some(family) = core
                .item
                .strip_prefix("arcadiandream:")
                .and_then(|s| s.strip_suffix("_bullet_core"))
            {
                if e.items[3]
                    .as_ref()
                    .is_some_and(|s| s.item.as_ref() == "arcadiandream:big_power_item")
                {
                    let result = ItemStack::new(&format!("arcadiandream:{family}_shot"), 1);
                    take(&mut e.items[0], 1);
                    take(&mut e.items[3], 1);
                    e.items[2] = Some(result);
                }
            }
            return;
        }
        if e.items[0].is_some() {
            return;
        }
        let Some(original) = e.items[1].as_ref().filter(|s| s.item.ends_with("_shot")) else {
            return;
        };
        let mut result = original.clone();
        let mut data = shot(&result);
        if !data.is_object() {
            return;
        }
        let mut used = [0u32; 7];
        if let Some(modifier) = &e.items[3] {
            let field = match modifier.item.as_ref() {
                "arcadiandream:power_item" => "power",
                "arcadiandream:big_power_item" => "density",
                "arcadiandream:point_item" => "speed",
                "arcadiandream:star_item" => "duration",
                _ => "",
            };
            if !field.is_empty() {
                let maxkey = match field {
                    "power" => "maxPower",
                    "density" => "maxDensity",
                    "speed" => "maxSpeed",
                    _ => "maxDuration",
                };
                let maximum = number(&data, maxkey, 100.0);
                for _ in 0..modifier.count {
                    let old = number(&data, field, 1.0);
                    if old >= maximum {
                        break;
                    }
                    let next = if field == "density" {
                        old + 1.0
                    } else {
                        let n = old * (1.0 + 0.05 * (1.0 - old / (maximum * 2.0)));
                        if field == "duration" { n.ceil() } else { n }
                    };
                    data[field] = json!(next.min(maximum));
                    data["cooldown"] = json!((number(&data, "cooldown", 5.0) + 1.0).min(number(
                        &data,
                        "maxCooldown",
                        100.0
                    )));
                    for (other, max, minimum) in [
                        ("power", "maxPower", 0.1),
                        ("speed", "maxSpeed", 0.05),
                        ("duration", "maxDuration", 20.0),
                    ] {
                        if other != field {
                            let v = number(&data, other, 1.0);
                            let n = (v * (1.0 - 0.02 * v / number(&data, max, 100.0))).max(minimum);
                            data[other] = json!(if other == "duration" { n.floor() } else { n });
                        }
                    }
                    used[3] += 1;
                }
            } else if modifier.item.as_ref() == "arcadiandream:time_orb"
                && data["timeOrbUsed"] != true
            {
                data["timeOrbUsed"] = json!(true);
                used[3] = 1;
            } else if modifier.item.as_ref() == "arcadiandream:enchanted_ice" && data["icy"] != true
            {
                data["icy"] = json!(true);
                data["divergence"] = json!((number(&data, "divergence", 0.0) + 5.0).min(number(
                    &data,
                    "maxDivergence",
                    90.0
                )));
                used[3] = 1;
            }
        }
        if let Some(repair) = e.items[4]
            .as_ref()
            .filter(|s| s.item.as_ref() == "arcadiandream:faith_item")
        {
            used[4] = result.damage.div_ceil(50).min(repair.count);
            result.damage = result.damage.saturating_sub(used[4] * 50);
        }
        if let Some(p) = &e.items[5] {
            if let Some(pattern) = p
                .item
                .strip_prefix("arcadiandream:")
                .and_then(|s| s.strip_suffix("_pattern"))
            {
                if ["spread", "ray", "ring", "arc", "double", "triple"].contains(&pattern) {
                    data["pattern"] = json!(pattern);
                    used[5] = 1;
                }
            }
        }
        if let Some(dye) = &e.items[6] {
            if let Some(color) = dye
                .item
                .strip_prefix("minecraft:")
                .and_then(|s| s.strip_suffix("_dye"))
            {
                if let Some(rgb) = dye_color(color) {
                    data["color"] = json!(rgb);
                    result.data_mut().dyed_color = Some(rgb as i32);
                    used[6] = 1;
                }
            }
        }
        if used.iter().all(|n| *n == 0) {
            return;
        }
        data["modified"] = json!(true);
        set_custom(&mut result, data);
        take(&mut e.items[1], 1);
        for i in 3..7 {
            take(&mut e.items[i], used[i]);
        }
        e.items[2] = Some(result);
    }
}
fn dye_color(s: &str) -> Option<u32> {
    Some(match s {
        "white" => 16777215,
        "orange" => 16753920,
        "magenta" => 16711935,
        "light_blue" => 3847130,
        "yellow" => 16770048,
        "lime" => 12582656,
        "pink" => 15961002,
        "gray" => 4210752,
        "light_gray" => 8421504,
        "cyan" => 65535,
        "purple" => 10494192,
        "blue" => 1908223,
        "brown" => 9849600,
        "green" => 65280,
        "red" => 16711680,
        "black" => 657930,
        _ => return None,
    })
}

pub fn update_tag(entity: &ModBlockEntity, out: &mut Option<Compound>) {
    if !pedestal(&entity.id) {
        return;
    }
    let mut tag = Compound::new();
    if let Some(stack) = entity.items.first().and_then(Option::as_ref) {
        tag.insert(
            "ArcadianDisplayItem".into(),
            horizon_client::save::item_to_nbt(stack),
        );
    } else {
        tag.insert(
            "ArcadianDisplayItem".into(),
            horizon_world::nbt::Tag::Compound(Compound::new()),
        );
    }
    *out = Some(tag);
}
impl Game<'_> {
    pub(super) fn arcadian_pedestal_draw(
        &self,
        entity: &ModBlockEntity,
        draw: &mut super::native_block_entity_draw::BlockEntityDraw,
    ) {
        if !pedestal(&entity.id) {
            return;
        }
        let stack = entity
            .nbt
            .get(&"ArcadianDisplayItem".into())
            .and_then(horizon_client::save::item_from_nbt)
            .or_else(|| entity.items.first().and_then(Option::as_ref).cloned());
        if let Some(stack) = stack {
            draw.push();
            let time = draw.game_time as f32 + draw.partial;
            draw.translate([0.5, 1.1 + (time * 0.1).sin() * 0.04, 0.5]);
            draw.rotate_about([0.0, 1.0, 0.0], time * 2.0, [0.0; 3]);
            let _ = self.native_draw_item(
                draw,
                &stack,
                horizon_assets::item_model::DisplayContext::Ground,
                None,
                0,
            );
            draw.pop();
        }
    }
}
