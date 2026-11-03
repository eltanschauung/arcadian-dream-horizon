// SPDX-License-Identifier: EUPL-1.2-or-later
// Native adaptation of Arcadian Dream by Maxmani and contributors.
use super::{Game, native_mobs::NativeSpawn};
use horizon_client::{items::ItemStack, text::Text};
use horizon_core::geometry::Vec3;
use horizon_world::nbt::Tag;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{cell::RefCell, collections::HashMap, path::PathBuf, sync::OnceLock};

pub fn definitions() -> &'static Value {
    static V: OnceLock<Value> = OnceLock::new();
    V.get_or_init(|| {
        serde_json::from_str(super::arcadian_assets::DATA).expect("embedded Arcadian data")
    })
}
pub fn custom(stack: &ItemStack) -> Value {
    fn json_tag(t: &Tag) -> Value {
        match t {
            Tag::Compound(c) => Value::Object(
                c.iter()
                    .map(|(k, v)| (k.to_string_lossy(), json_tag(v)))
                    .collect(),
            ),
            Tag::String(s) => json!(s.to_string_lossy()),
            Tag::Int(n) => json!(*n),
            Tag::Byte(n) => json!(*n != 0),
            Tag::Long(n) => json!(*n),
            Tag::Float(n) => json!(f32::from_bits(*n)),
            Tag::Double(n) => json!(f64::from_bits(*n)),
            _ => Value::Null,
        }
    }
    stack
        .data()
        .and_then(|d| d.other.iter().find(|(k, _)| k == "minecraft:custom_data"))
        .map(|(_, v)| json_tag(v)["arcadiandream"].clone())
        .filter(|v| v.is_object())
        .unwrap_or_else(|| json!({}))
}
pub fn set_custom(stack: &mut ItemStack, data: Value) {
    // Preserve other mods' namespaced data rather than replacing the entire component.
    let mut full = stack
        .data()
        .and_then(|d| d.other.iter().find(|(k, _)| k == "minecraft:custom_data"))
        .and_then(|(_, v)| {
            if let Tag::Compound(c) = v {
                Some(c.clone())
            } else {
                None
            }
        })
        .unwrap_or_default();
    full.insert(
        "arcadiandream".into(),
        horizon_client::lan::data::json_to_nbt(&data),
    );
    let d = stack.data_mut();
    d.other.retain(|(k, _)| k != "minecraft:custom_data");
    d.other
        .push(("minecraft:custom_data".into(), Tag::Compound(full)));
}
pub fn shot(stack: &ItemStack) -> Value {
    let family = stack
        .item
        .strip_prefix("arcadiandream:")
        .unwrap_or("")
        .trim_end_matches("_shot");
    let mut v = definitions()["shots"][family].clone();
    if let (Some(base), Some(own)) = (v.as_object_mut(), custom(stack).as_object()) {
        base.extend(own.clone());
    }
    v
}
pub fn number(v: &Value, k: &str, default: f64) -> f64 {
    v[k].as_f64().filter(|n| n.is_finite()).unwrap_or(default)
}
#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(default)]
struct PlayerData {
    uuid: Option<[i32; 4]>,
    elixir: u8,
    ethereal: u32,
    fatigue: u32,
    fatigue_level: u8,
    charm: i32,
    position: [f64; 3],
    dimension: String,
    accessories: HashMap<String, Vec<u8>>,
    tiny: bool,
}
#[derive(Default)]
struct State {
    world: Option<PathBuf>,
    players: HashMap<String, PlayerData>,
    tick: u64,
    queue: Vec<String>,
    dirty: bool,
    sent: HashMap<u128, HashMap<u32, u64>>,
}
thread_local! {static STATE:RefCell<State>=RefCell::new(State::default());}
fn state<T>(world: Option<PathBuf>, f: impl FnOnce(&mut State) -> T) -> T {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        if s.world != world {
            save(&s);
            s.players = world
                .as_ref()
                .and_then(|p| std::fs::read(p.join("arcadiandream/players.json")).ok())
                .and_then(|b| serde_json::from_slice(&b).ok())
                .unwrap_or_default();
            horizon_client::projectiles::arcadian_motion::clear();
            s.world = world;
            s.tick = 0;
            s.queue.clear();
            s.sent.clear();
            s.dirty = false;
        }
        f(&mut s)
    })
}
fn save(s: &State) {
    if !s.dirty {
        return;
    }
    if let Some(root) = &s.world {
        let dir = root.join("arcadiandream");
        let result = (|| -> std::io::Result<()> {
            std::fs::create_dir_all(&dir)?;
            let bytes = serde_json::to_vec_pretty(&s.players)?;
            std::fs::write(dir.join("players.json.tmp"), bytes)?;
            std::fs::rename(dir.join("players.json.tmp"), dir.join("players.json"))
        })();
        if let Err(e) = result {
            eprintln!("Arcadian Dream: saving player state: {e}");
        }
    }
}
pub fn config() -> &'static Value {
    horizon_world::loot::arcadian_settings::config()
}

fn dist(a: [f64; 3], b: [f64; 3]) -> f64 {
    (0..3).map(|i| (a[i] - b[i]).powi(2)).sum::<f64>().sqrt()
}
pub(super) fn uuid(id: u128) -> String {
    let s = format!("{id:032x}");
    format!(
        "{}-{}-{}-{}-{}",
        &s[..8],
        &s[8..12],
        &s[12..16],
        &s[16..20],
        &s[20..]
    )
}
impl Game<'_> {
    fn arcadian_note(&mut self, message: &str) {
        let id = self.native_acting_id();
        self.native_tell(id, Text::literal(message));
    }
    pub(super) fn arcadian_queue(&mut self, command: String) {
        state(self.native_world_dir(), |s| s.queue.push(command));
    }
    fn arcadian_tp(&mut self, pos: [f64; 3]) {
        let id = uuid(self.native_acting_id());
        self.arcadian_queue(format!(
            "execute as {id} at @s run tp @s {} {} {}",
            pos[0], pos[1], pos[2]
        ));
    }
    pub(super) fn arcadian_use_on(
        &mut self,
        stack: &ItemStack,
        pos: [i32; 3],
        face: [i32; 3],
    ) -> bool {
        if !self.native_on("arcadiandream") {
            return false;
        }
        if let Some(name) = stack
            .item
            .strip_prefix("arcadiandream:")
            .and_then(|n| n.strip_suffix("_spawn_egg"))
        {
            let at = std::array::from_fn(|i| {
                f64::from(pos[i] + face[i]) + if i == 1 { 0.0 } else { 0.5 }
            });
            if self
                .native_spawn_mob(
                    self.dim.id(),
                    &format!("arcadiandream:{name}"),
                    at,
                    &NativeSpawn {
                        no_ai: true,
                        no_gravity: true,
                        ..Default::default()
                    },
                )
                .is_some()
            {
                self.consume_held();
            }
            return true;
        }
        if stack.item.as_ref() == "arcadiandream:wall_passing_chisel" {
            if !config()["chiselOptions"]["canUse"]
                .as_bool()
                .unwrap_or(true)
            {
                return true;
            }
            let max = config()["chiselOptions"]["maxDistance"]
                .as_i64()
                .unwrap_or(16)
                .clamp(1, 64) as i32;
            for n in 1..=max {
                let p = std::array::from_fn(|i| pos[i] - face[i] * n);
                let block = self.native_block(p);
                if block == "minecraft:obsidian" || block == "minecraft:bedrock" {
                    break;
                }
                if matches!(block.as_str(), "minecraft:air" | "minecraft:cave_air")
                    && self.native_block([p[0], p[1] + 1, p[2]]) == "minecraft:air"
                {
                    self.arcadian_tp([p[0] as f64 + 0.5, p[1] as f64, p[2] as f64 + 0.5]);
                    self.damage_held(1);
                    break;
                }
            }
            return true;
        }
        if stack.item.as_ref() == "arcadiandream:ibuki_gourd"
            && self.native_block(pos) == "minecraft:water"
        {
            if let Some(s) = self.held_mut() {
                let mut d = custom(s);
                d["filled"] = json!(true);
                set_custom(s, d);
            }
            return true;
        }
        self.arcadian_use(stack)
    }
    pub(super) fn arcadian_use(&mut self, stack: &ItemStack) -> bool {
        if !self.native_on("arcadiandream") {
            return false;
        }
        let Some(item) = stack.item.strip_prefix("arcadiandream:") else {
            return false;
        };
        if item.ends_with("_shot") {
            self.arcadian_fire(stack);
            return true;
        }
        let key = uuid(self.native_acting_id());
        let world = self.native_world_dir();
        match item {
            "fairy_charm" => {
                if self.keys.contains(&winit::keyboard::KeyCode::ShiftLeft) {
                    let mode = (custom(stack)["mode"].as_i64().unwrap_or(0) + 1) % 3;
                    if let Some(s) = self.held_mut() {
                        let mut d = custom(s);
                        d["mode"] = json!(mode);
                        set_custom(s, d);
                    }
                    self.add_cooldown(stack.item.clone(), 20);
                    self.arcadian_note(
                        [
                            "Fairy Charm: pacify",
                            "Fairy Charm: prevent spawning",
                            "Fairy Charm: pacify and prevent spawning",
                        ][mode as usize],
                    );
                    true
                } else {
                    self.arcadian_equip(stack, "charm")
                }
            }
            "magatama_necklace" => self.arcadian_equip(stack, "necklace"),
            "extend_item" => self.arcadian_equip(stack, "life"),
            "healing_charm" => false,
            "ghastly_lantern" => {
                state(world, |s| {
                    s.players.entry(key).or_default().ethereal = 300;
                    s.dirty = true;
                });
                self.add_player_effect(horizon_client::effects::EffectInstance::new(
                    "arcadiandream:ethereal",
                    300,
                    0,
                ));
                self.consume_held();
                self.add_cooldown(stack.item.clone(), 300);
                true
            }
            "hourai_elixir" => {
                if !config()["houraiElixirOptions"]["canDrink"]
                    .as_bool()
                    .unwrap_or(false)
                {
                    self.arcadian_note(
                        "Drinking the Hourai Elixir is disabled in the server config.",
                    );
                    true
                } else {
                    false
                }
            }
            "ibuki_gourd" => !custom(stack)["filled"].as_bool().unwrap_or(false),
            "bomb_item" => {
                let (eye, _) = self.aim().unwrap_or(([0.0; 3], [0.0, 0.0, 1.0]));
                let own = self.acting_player_uuid();
                let count = self
                    .creatures
                    .projectiles
                    .iter()
                    .filter(|p| {
                        p.kind.id().starts_with("arcadiandream:")
                            && p.owner_uuid != Some(own)
                            && dist([p.position.x, p.position.y, p.position.z], eye) < 24.0
                    })
                    .count();
                if count > 0 {
                    self.arcadian_cancelled(count, false);
                    self.creatures.projectiles.retain(|p| {
                        !p.kind.id().starts_with("arcadiandream:")
                            || p.owner_uuid == Some(own)
                            || dist([p.position.x, p.position.y, p.position.z], eye) >= 24.0
                    });
                    self.consume_held();
                    let mut left = count;
                    while left > 0 {
                        let n = left.min(64) as u32;
                        self.give_or_drop(ItemStack::new("arcadiandream:star_item", n));
                        left -= n as usize;
                    }
                    self.add_cooldown(stack.item.clone(), 100);
                }
                count > 0
            }
            "death_scythe" => false,
            "miracle_mallet" => {
                if self.keys.contains(&winit::keyboard::KeyCode::ShiftLeft) {
                    state(world, |s| {
                        let p = s.players.entry(key).or_default();
                        p.tiny = !p.tiny;
                        s.dirty = true;
                        save(s);
                    });
                    self.damage_held(1);
                    self.add_cooldown(stack.item.clone(), 60);
                    return true;
                }
                if let Some((eye, dir)) = self.aim() {
                    let own = self.acting_player_uuid();
                    let target = self
                        .creatures
                        .projectiles
                        .iter()
                        .filter(|p| {
                            p.kind.id().starts_with("arcadiandream:") && p.owner_uuid == Some(own)
                        })
                        .filter_map(|p| {
                            let pos = [p.position.x, p.position.y, p.position.z];
                            let t = (0..3).map(|i| (pos[i] - eye[i]) * dir[i]).sum::<f64>();
                            let off = (0..3)
                                .map(|i| (pos[i] - eye[i] - dir[i] * t).powi(2))
                                .sum::<f64>();
                            (t > 0.0 && t < 16.0 && off < 1.0).then_some((p.id, t))
                        })
                        .min_by(|a, b| a.1.total_cmp(&b.1));
                    if let Some((id, _)) = target {
                        horizon_client::projectiles::arcadian_motion::enlarge(id);
                        self.damage_held(1);
                        self.add_cooldown(stack.item.clone(), 60);
                    }
                }
                true
            }
            _ if item.starts_with("hihiirokane_") => {
                if self.offhand.is_none()
                    && self.keys.contains(&winit::keyboard::KeyCode::ShiftLeft)
                {
                    if let Some(s) = self.held_mut() {
                        let mut d = custom(s);
                        d["autosmelt"] = json!(!custom(s)["autosmelt"].as_bool().unwrap_or(true));
                        set_custom(s, d);
                    }
                    self.arcadian_note("Hihiirokane tool mode switched.");
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }
    pub(super) fn arcadian_release(&mut self, stack: &ItemStack, time_left: i32) -> bool {
        if stack.item.as_ref() != "arcadiandream:death_scythe"
            || !self.native_on("arcadiandream")
            || self.remote.is_some()
            || self.options.execution_role.is_client()
            || 72000 - time_left <= 20
        {
            return false;
        }
        if let Some((eye, dir)) = self.aim() {
            let reach = self
                .clip_ray(eye, dir, 16.0)
                .map(|h| dist(h.point, eye))
                .unwrap_or(16.0);
            let mut candidates = self
                .mobs
                .iter()
                .filter(|m| m.alive() && m.kind.id != "minecraft:player")
                .filter_map(|m| {
                    super::ray_box(eye, dir, reach, m.bounds()).map(|t| {
                        (
                            t,
                            m.id,
                            None,
                            [m.position.x, m.position.y, m.position.z],
                            true,
                        )
                    })
                })
                .collect::<Vec<_>>();
            let own = self.native_acting_id();
            let dimension = self.dim.id().to_owned();
            for p in self
                .native_server_players()
                .into_iter()
                .filter(|p| p.uuid != own && p.dimension == dimension && p.game_mode != 3)
            {
                let [x, y, z] = p.position;
                let bounds = horizon_core::geometry::Aabb::new(
                    Vec3::new(x - 0.3, y, z - 0.3),
                    Vec3::new(x + 0.3, y + 1.8, z + 0.3),
                );
                if let Some(t) = super::ray_box(eye, dir, reach, bounds) {
                    candidates.push((
                        t,
                        0,
                        Some(p.uuid),
                        p.position,
                        p.game_mode != 1 && self.ui.rules.bool("pvp"),
                    ));
                }
            }
            if let Some((_, mob, other, to, allowed)) =
                candidates.into_iter().min_by(|a, b| a.0.total_cmp(&b.0))
            {
                if allowed {
                    if let Some(player) = self.native_player() {
                        let from = [player.x, player.y, player.z];
                        if let Some(id) = other {
                            self.arcadian_queue(format!(
                                "execute in {dimension} run tp {} {} {} {}",
                                uuid(id),
                                from[0],
                                from[1],
                                from[2]
                            ));
                        } else {
                            let _ = self.native_teleport_mob(&dimension, mob, from);
                        }
                        self.arcadian_tp(to);
                        self.damage_held(1);
                        self.add_cooldown(stack.item.clone(), 100);
                        self.audio.play(
                            "arcadiandream:item.death_scythe.teleport",
                            Some(to),
                            1.0,
                            1.1,
                        );
                        self.stats.add("used", "arcadiandream:death_scythe", 1);
                    }
                }
            }
        }
        true
    }
    pub(super) fn arcadian_finish(&mut self, stack: &ItemStack, result: &mut Option<ItemStack>) {
        if !self.native_on("arcadiandream") {
            return;
        }
        match stack.item.as_ref() {
            "arcadiandream:healing_charm" => {
                self.give_player_effect("regeneration", 100, 1);
            }
            "arcadiandream:ibuki_gourd" => {
                let mut out = stack.clone();
                let mut d = custom(stack);
                if d["filled"] == true {
                    self.give_player_effect("strength", 400, 0);
                    self.give_player_effect("nausea", 600, 0);
                }
                d["filled"] = json!(false);
                set_custom(&mut out, d);
                *result = Some(out);
            }
            "arcadiandream:hourai_elixir" => {
                if config()["houraiElixirOptions"]["canDrink"] != true {
                    return;
                }
                let key = uuid(self.native_acting_id());
                let count = state(self.native_world_dir(), |s| {
                    let p = s.players.entry(key).or_default();
                    p.elixir = (p.elixir + 1).min(3);
                    let count = p.elixir;
                    s.dirty = true;
                    save(s);
                    count
                });
                for effect in ["blindness", "weakness", "poison"] {
                    self.give_player_effect(effect, 100, 0);
                }
                let mut out = stack.clone();
                if !self.creative {
                    out.damage += 1;
                }
                *result = if out.damage >= 3 { None } else { Some(out) };
                self.arcadian_note(&format!("Hourai Elixir: {count}/3 doses"));
            }
            _ => {}
        }
    }
    pub(super) fn arcadian_owner_ethereal(&mut self, owner: [i32; 4]) -> bool {
        let id = uuid(horizon_client::lan::playerdata::ints_uuid(owner));
        state(self.native_world_dir(), |s| {
            s.players.get(&id).is_some_and(|p| p.ethereal > 0)
        })
    }
    pub(super) fn arcadian_ethereal(&mut self) -> bool {
        let id = uuid(self.native_acting_id());
        self.potions.effects.get("arcadiandream:ethereal").is_some()
            || state(self.native_world_dir(), |s| {
                s.players.get(&id).is_some_and(|p| p.ethereal > 0)
            })
    }
    pub(super) fn arcadian_fire(&mut self, stack: &ItemStack) {
        use horizon_client::{
            mobs::Who,
            projectiles::{Projectile, ProjectileKind, arcadian_motion, thrown_type},
        };
        let data = shot(stack);
        if !data.is_object() {
            return;
        }
        let durability = number(&data, "durability", 100.0) as u32;
        if stack.damage >= durability.saturating_sub(1) {
            return;
        }
        let Some((eye, aim)) = self.aim() else { return };
        let family = stack
            .item
            .trim_start_matches("arcadiandream:")
            .trim_end_matches("_shot");
        let Some(base) = thrown_type(&format!("arcadiandream:{family}_bullet")) else {
            return;
        };
        let power = number(&data, "power", 2.0)
            * number(config(), "danmakuDamageMultiplier", 1.0).clamp(1.0, 4.0);
        let t = arcadian_motion::damage_type(base, power as f32);
        let density = number(&data, "density", 1.0).clamp(1.0, 40.0) as usize;
        let player = self.native_acting_id();
        if density >= 25 {
            self.native_trigger_criterion(player, "arcadiandream:curtain_fire", |c| {
                density as u64 >= c.conditions["density"].as_u64().unwrap_or(25)
            });
        }
        let pattern = data["pattern"].as_str().unwrap_or("spread");
        let speed = number(&data, "speed", 0.8);
        for (n, (angle, layer)) in directions(pattern, density).into_iter().enumerate() {
            let spread = number(&data, "divergence", 0.0)
                + if pattern == "spread" {
                    density.saturating_sub(1) as f64
                } else {
                    0.0
                };
            let noise =
                ((self.native_day_time() as f64 + n as f64 * 13.1).sin() * spread).to_radians();
            let a = angle + noise;
            let dir = [
                aim[0] * a.cos() - aim[2] * a.sin(),
                aim[1],
                aim[0] * a.sin() + aim[2] * a.cos(),
            ];
            let mut p = Projectile::new(
                ProjectileKind::Thrown(t),
                Vec3::new(eye[0], eye[1] - 0.1, eye[2]),
                Some(Who::Player),
            );
            p.guest = self.lan_acting;
            p.no_gravity = true;
            p.velocity = Vec3::new(
                dir[0] * speed * (1.0 - layer as f64 / density as f64).max(0.05),
                dir[1] * speed * (1.0 - layer as f64 / density as f64).max(0.05),
                dir[2] * speed * (1.0 - layer as f64 / density as f64).max(0.05),
            );
            // add_projectile assigns its network identity before recording per-shot state.
            self.add_projectile(p);
            if let Some(p) = self.creatures.projectiles.last() {
                arcadian_motion::register(p.id, data.clone());
            }
        }
        let cooldown = ((number(&data, "cooldown", 5.0)
            * number(config(), "danmakuCooldownMultiplier", 1.0).clamp(1.0, 4.0))
            / if data["timeOrbUsed"] == true {
                2.0
            } else {
                1.0
            }) as u32;
        if config()["cooldownPerBulletType"].as_bool().unwrap_or(true) {
            self.add_cooldown(stack.item.clone(), cooldown as i32);
        } else {
            for f in ["circle", "bubble", "amulet", "star", "kunai", "pellet"] {
                self.add_cooldown(format!("arcadiandream:{f}_shot").into(), cooldown as i32);
            }
        }
        if !self.creative {
            if let Some(s) = self.held_mut() {
                s.damage = (s.damage + 1 + density as u32 / 4).min(durability - 1);
            }
        }
    }
    pub(super) fn arcadian_player_tick(&mut self) {
        if !self.native_on("arcadiandream") {
            return;
        }
        let id = uuid(self.native_acting_id());
        let world = self.native_world_dir();
        let charm = state(world.clone(), |s| {
            s.players
                .get(&id)
                .and_then(|p| p.accessories.get("charm"))
                .and_then(|b| horizon_world::nbt::read_network(b, Default::default()).ok())
                .and_then(|(t, _)| horizon_client::save::item_from_nbt(&t))
                .map(|s| custom(&s)["mode"].as_i64().unwrap_or(0) as i32)
                .unwrap_or(-1)
        });
        let pos = self
            .native_player()
            .map(|p| [p.x, p.y, p.z])
            .unwrap_or([0.0; 3]);
        let dim = self.dim.id().to_string();
        let owner = self.acting_player_uuid();
        let effect_state = state(world, |s| {
            let p = s.players.entry(id).or_default();
            p.uuid = Some(owner);
            p.ethereal = p.ethereal.saturating_sub(1);
            p.fatigue = p.fatigue.saturating_sub(1);
            p.charm = charm;
            p.position = pos;
            p.dimension = dim;
            let out = p.clone();
            s.dirty = true;
            out
        });
        let mut modified = false;
        let mut kills = 0;
        for stack in self
            .inventory
            .iter_mut()
            .chain(std::iter::once(&mut self.offhand))
            .flatten()
        {
            if stack.item.starts_with("arcadiandream:") && stack.item.ends_with("_shot") {
                let mut d = custom(stack);
                if d["modified"] == true {
                    modified = true;
                    d.as_object_mut().unwrap().remove("modified");
                    set_custom(stack, d);
                }
            }
            if stack.item.as_ref() == "arcadiandream:mochi_mallet" {
                kills = kills.max(custom(stack)["kill_count"].as_u64().unwrap_or(0));
            }
        }
        let player = self.native_acting_id();
        if modified {
            self.native_trigger_criterion(player, "arcadiandream:danmaku_modified", |_| true);
        }
        if kills >= 100 {
            self.native_trigger_criterion(player, "arcadiandream:mochi_mallet_kills", |c| {
                kills >= c.conditions["kills"].as_u64().unwrap_or(100)
            });
        }
        self.arcadian_player_effects(&effect_state);
        self.arcadian_equipment_tick();
    }
    pub(super) fn arcadian_hurt(&mut self, _amount: f32, damage_type: &str) -> Option<bool> {
        if !self.native_on("arcadiandream")
            || damage_type.ends_with("out_of_world")
            || damage_type.ends_with("generic_kill")
        {
            return None;
        }
        self.arcadian_ethereal().then_some(false)
    }
    fn arcadian_protection(&mut self) -> Option<bool> {
        let id = uuid(self.native_acting_id());
        state(self.native_world_dir(), |s| {
            let p = s.players.entry(id).or_default();
            if p.accessories.contains_key("life") {
                Some(false)
            } else if p.elixir >= 3
                && (!config()["houraiElixirOptions"]["allowDying"]
                    .as_bool()
                    .unwrap_or(false)
                    || p.fatigue == 0
                    || p.fatigue_level + 1
                        < config()["houraiElixirOptions"]["elixirFatigueThreshold"]
                            .as_u64()
                            .unwrap_or(1) as u8)
            {
                Some(true)
            } else {
                None
            }
        })
    }
    pub(super) fn arcadian_totem_ready(&mut self) -> bool {
        if !self.native_on("arcadiandream") {
            return false;
        }
        let native = self.arcadian_protection().is_some();
        if native {
            if let Some(p) = self.player.as_mut() {
                p.health.totem_ready = true;
            }
        }
        native
    }
    pub(super) fn arcadian_resurrect(&mut self) -> Option<([f64; 3], f64, f64)> {
        if !self.player.as_ref()?.health.totem_used {
            return None;
        }
        let immortal = self.arcadian_protection()?;
        let id = uuid(self.native_acting_id());
        let world = self.native_world_dir();
        let p = self.player.as_mut()?;
        p.health.totem_used = false;
        let max = p
            .attributes
            .key("minecraft:max_health")
            .and_then(|k| {
                p.attributes
                    .value(Some(k), horizon_world::attribute_map::ValueKind::Computed)
                    .ok()
            })
            .unwrap_or(20.0) as f32;
        let _ = p.set_health(max, false);
        let body = (
            [p.position.x, p.position.y, p.position.z],
            f64::from(p.dimensions.width),
            f64::from(p.dimensions.height),
        );
        if immortal {
            if config()["houraiElixirOptions"]["nerfElixir"]
                .as_bool()
                .unwrap_or(true)
            {
                let level = state(world, |s| {
                    let p = s.players.entry(id).or_default();
                    p.fatigue_level = if p.fatigue > 0 {
                        (p.fatigue_level + 1).min(4)
                    } else {
                        0
                    };
                    p.fatigue = 6000;
                    let l = p.fatigue_level;
                    s.dirty = true;
                    save(s);
                    l
                });
                self.add_player_effect(horizon_client::effects::EffectInstance::new(
                    "arcadiandream:elixir_fatigue",
                    6000,
                    level as i32,
                ));
            }
        } else {
            state(world, |s| {
                s.players.entry(id).or_default().accessories.remove("life");
                s.dirty = true;
                save(s);
            });
            self.potions.effects.clear();
            self.give_player_effect("resistance", 60, 4);
            let own = self.acting_player_uuid();
            let before = self.creatures.projectiles.len();
            self.creatures.projectiles.retain(|p| {
                !p.kind.id().starts_with("arcadiandream:")
                    || p.owner_uuid == Some(own)
                    || dist([p.position.x, p.position.y, p.position.z], body.0) > 24.0
            });
            let count = before - self.creatures.projectiles.len();
            self.arcadian_cancelled(count, true);
        }
        Some(body)
    }
    fn arcadian_equip(&mut self, stack: &ItemStack, slot: &str) -> bool {
        let id = uuid(self.native_acting_id());
        let world = self.native_world_dir();
        let old = state(world.clone(), |s| {
            s.players
                .entry(id.clone())
                .or_default()
                .accessories
                .get(slot)
                .cloned()
        });
        if old.is_some() && self.creative && !self.inventory.iter().any(Option::is_none) {
            self.arcadian_note("Make an empty inventory slot before swapping accessories.");
            return true;
        }
        let mut one = stack.clone();
        one.count = 1;
        let Ok(bytes) = horizon_world::nbt::write_network(
            &horizon_client::save::item_to_nbt(&one),
            Default::default(),
        ) else {
            return true;
        };
        self.consume_held();
        state(world, |s| {
            s.players
                .entry(id)
                .or_default()
                .accessories
                .insert(slot.into(), bytes);
            s.dirty = true;
            save(s);
        });
        if let Some(old) = old
            .and_then(|b| horizon_world::nbt::read_network(&b, Default::default()).ok())
            .and_then(|(t, _)| horizon_client::save::item_from_nbt(&t))
        {
            let player = self.native_acting_id();
            let _ = self.native_give_stack(player, old);
        }
        self.arcadian_note(&format!(
            "Equipped {slot} accessory. /arcadian unequip {slot} returns it."
        ));
        true
    }
    pub(super) fn arcadian_command(&mut self, player: u128, command: &str) -> bool {
        let Some(arg) = command.strip_prefix("arcadian unequip ") else {
            return false;
        };
        let slot = arg.trim();
        if !["life", "charm", "necklace"].contains(&slot) {
            self.native_tell(player, Text::literal("Choose life, charm or necklace."));
            return true;
        }
        let id = uuid(player);
        let world = self.native_world_dir();
        let old = state(world.clone(), |s| {
            s.players
                .get(&id)
                .and_then(|p| p.accessories.get(slot).cloned())
        });
        if let Some(stack) = old
            .and_then(|b| horizon_world::nbt::read_network(&b, Default::default()).ok())
            .and_then(|(t, _)| horizon_client::save::item_from_nbt(&t))
        {
            if self.native_give_stack(player, stack).is_none() {
                state(world, |s| {
                    s.players.entry(id).or_default().accessories.remove(slot);
                    s.dirty = true;
                    save(s);
                });
            } else {
                self.native_tell(player, Text::literal("Make room in your inventory first."));
            }
        } else {
            self.native_tell(player, Text::literal("That accessory slot is empty."));
        }
        true
    }
    pub(super) fn arcadian_cancelled(&mut self, count: usize, from_extend: bool) {
        if count == 0 {
            return;
        }
        self.stats.add(
            "custom",
            "arcadiandream:bullets_cancelled",
            count.min(i32::MAX as usize) as i32,
        );
        let id = self.native_acting_id();
        self.native_trigger_criterion(id, "arcadiandream:bullets_cancelled", |c| {
            let v = c.conditions;
            let min = v["amount"]["min"].as_u64().unwrap_or(0);
            let max = v["amount"]["max"].as_u64().unwrap_or(u64::MAX);
            (count as u64) >= min
                && (count as u64) <= max
                && v["from_extend"].as_bool().is_none_or(|b| b == from_extend)
        });
    }
    pub(super) fn arcadian_hit(&mut self, stack: &ItemStack, entity: u32, _kind: &str) {
        if !self.native_on("arcadiandream") {
            return;
        }
        self.arcadian_special_melee(stack, entity);
        match stack.item.as_ref() {
            "arcadiandream:folding_chair" => {
                let from = self.native_player().map(|p| [p.x, p.z]);
                if let (Some(from), Some(m)) = (from, self.mobs.iter_mut().find(|m| m.id == entity))
                {
                    let dx = m.position.x - from[0];
                    let dz = m.position.z - from[1];
                    let d = (dx * dx + dz * dz).sqrt().max(0.01);
                    let k = number(&config()["foldingChairOptions"], "knockbackStrength", 1.5)
                        .clamp(0.0, 10.0);
                    m.velocity = Vec3::new(dx / d * k, 0.4, dz / d * k);
                }
            }
            "arcadiandream:icicle_sword" => {
                if let Some(m) = self.mobs.iter_mut().find(|m| m.id == entity) {
                    m.ticks_frozen = (m.ticks_frozen + 40).min(560);
                }
            }
            _ => {}
        }
        if let Some(m) = self
            .mobs
            .iter_mut()
            .find(|m| m.id == entity && m.kind.id.starts_with("arcadiandream:"))
        {
            m.extra.counter = 600;
        }
    }
    pub(super) fn arcadian_world_tick(&mut self, dimension: &str) {
        if !self.native_on("arcadiandream") {
            return;
        }
        self.arcadian_fairies(dimension);
    }
    pub(super) fn arcadian_receive(&mut self, channel: &str, bytes: &[u8]) {
        if channel == "arcadiandream:player" && bytes.len() < 30000 {
            if let Ok(p) = serde_json::from_slice::<PlayerData>(bytes) {
                *client_player().lock().unwrap() = p;
            }
            return;
        }
        if channel != "arcadiandream:entities" || bytes.len() > 30000 {
            return;
        }
        let Ok(v) = serde_json::from_slice::<Value>(bytes) else {
            return;
        };
        super::arcadian_fairies::received(&v["fairies"]);
        if let Some(rows) = v["shots"].as_array() {
            for row in rows {
                if let Some(id) = row[0].as_u64() {
                    horizon_client::projectiles::arcadian_motion::register(
                        id as u32,
                        row[1].clone(),
                    );
                }
            }
        }
    }
    fn arcadian_sync(&mut self, dimension: &str) {
        if self.world.game_time.rem_euclid(2) != 0 {
            return;
        }
        use std::hash::{Hash, Hasher};
        let ids = self
            .mobs
            .iter()
            .filter(|m| m.kind.id.starts_with("arcadiandream:"))
            .map(|m| {
                (
                    m.id,
                    m.kind.id.clone(),
                    [m.position.x, m.position.y, m.position.z],
                )
            })
            .collect::<Vec<_>>();
        let mut rows = Vec::new();
        for (id, kind, at) in ids {
            rows.push((
                id,
                at,
                true,
                json!([id, self.arcadian_fairy_data(id, &kind)]),
            ));
        }
        for p in self
            .creatures
            .projectiles
            .iter()
            .filter(|p| p.kind.id().starts_with("arcadiandream:"))
        {
            rows.push((p.id, [p.position.x, p.position.y, p.position.z], false, {
                let mut data = horizon_client::projectiles::arcadian_motion::data(p.id);
                data["owner"] = json!(p.owner_uuid);
                json!([p.id, data])
            }));
        }
        let world = self.native_world_dir();
        for player in self
            .native_server_players()
            .into_iter()
            .filter(|p| p.dimension == dimension)
        {
            let mut nearby = rows
                .iter()
                .filter(|(_, at, _, _)| dist(*at, player.position) < 128.0)
                .collect::<Vec<_>>();
            nearby
                .sort_by(|a, b| dist(a.1, player.position).total_cmp(&dist(b.1, player.position)));
            let payloads = state(world.clone(), |s| {
                let sent = s.sent.entry(player.uuid).or_default();
                let live = nearby
                    .iter()
                    .map(|r| r.0)
                    .collect::<std::collections::HashSet<_>>();
                sent.retain(|id, _| live.contains(id));
                nearby
                    .into_iter()
                    .filter_map(|r| {
                        let bytes = serde_json::to_vec(&r.3).unwrap();
                        let mut h = std::collections::hash_map::DefaultHasher::new();
                        bytes.hash(&mut h);
                        let hash = h.finish();
                        (sent.get(&r.0) != Some(&hash)).then_some((r.0, r.2, r.3.clone(), hash))
                    })
                    .collect::<Vec<_>>()
            });
            for part in payloads.chunks(60).take(30) {
                let message = json!({"fairies":part.iter().filter(|r|r.1).map(|r|&r.2).collect::<Vec<_>>(),"shots":part.iter().filter(|r|!r.1).map(|r|&r.2).collect::<Vec<_>>()});
                let bytes = serde_json::to_vec(&message).unwrap();
                if bytes.len() < 30000
                    && self
                        .native_net_send_to_player(player.uuid, "arcadiandream:entities", &bytes)
                        .is_ok()
                {
                    state(world.clone(), |s| {
                        let sent = s.sent.entry(player.uuid).or_default();
                        for r in part {
                            sent.insert(r.0, r.3);
                        }
                    });
                }
            }
            if self.world.game_time.rem_euclid(10) == 0 {
                let data = state(world.clone(), |s| {
                    s.players.get(&uuid(player.uuid)).cloned()
                });
                if let Some(data) = data {
                    if let Ok(bytes) = serde_json::to_vec(&data) {
                        let _ = self.native_net_send_to_player(
                            player.uuid,
                            "arcadiandream:player",
                            &bytes,
                        );
                    }
                }
            }
        }
    }
    pub(super) fn arcadian_flush(&mut self, dimension: &str) {
        if !self.native_on("arcadiandream") {
            return;
        }
        self.arcadian_sync(dimension);
        let queue = state(self.native_world_dir(), |s| {
            s.tick += 1;
            if s.tick % 100 == 0 {
                save(s);
                s.dirty = false;
            }
            std::mem::take(&mut s.queue)
        });
        for c in queue {
            let _ = self.native_run_command(&c);
        }
    }
    fn arcadian_fairies(&mut self, dimension: &str) {
        let time = self.native_day_time();
        let players = self
            .native_server_players()
            .into_iter()
            .filter(|p| p.dimension == dimension && p.game_mode != 1 && p.game_mode != 3)
            .collect::<Vec<_>>();
        let fairy_ids = self
            .mobs
            .iter()
            .filter(|m| m.kind.id.starts_with("arcadiandream:") && m.alive())
            .map(|m| m.id)
            .collect::<Vec<_>>();
        let charms = state(self.native_world_dir(), |s| {
            s.players.values().cloned().collect::<Vec<_>>()
        });
        let mut shots = Vec::new();
        for id in &fairy_ids {
            let Some(kind) = self
                .mobs
                .iter()
                .find(|m| m.id == *id)
                .map(|m| m.kind.id.clone())
            else {
                continue;
            };
            let mut data = self.arcadian_fairy_data(*id, &kind);
            let spell = super::arcadian_fairies::attack(&kind, &data);
            let Some(m) = self.mobs.iter_mut().find(|m| m.id == *id) else {
                continue;
            };
            m.no_ai = true;
            m.no_gravity = true;
            let pos = [m.position.x, m.position.y, m.position.z];
            let sunflower = kind.ends_with(":sunflower_fairy");
            if m.hurt_time > 0 {
                data.anger = 400 + (m.id % 400);
            }
            data.anger = data.anger.saturating_sub(1);
            data.cooldown -= 1;
            let target = players
                .iter()
                .filter(|p| {
                    data.anger > 0
                        || !charms.iter().any(|c| {
                            c.dimension == dimension
                                && c.charm >= 0
                                && c.charm != 1
                                && dist(c.position, p.position) < 2.0
                        })
                })
                .filter(|p| dist(p.position, pos) < 24.0)
                .min_by(|a, b| dist(a.position, pos).total_cmp(&dist(b.position, pos)));
            if let Some(p) = target.filter(|_| sunflower || data.anger > 0) {
                let d = dist(pos, p.position).max(0.01);
                m.yaw = (-(p.position[0] - pos[0]))
                    .atan2(p.position[2] - pos[2])
                    .to_degrees() as f32;
                m.body_yaw = m.yaw;
                m.head_yaw = m.yaw;
                if data.cooldown <= 0 {
                    shots.push((
                        m.id,
                        pos,
                        p.position,
                        kind.clone(),
                        data.bullet,
                        spell.clone(),
                    ));
                    data.cooldown = number(&spell, "cooldown", 30.0).max(1.0) as i32;
                }
                let factor = if d > 10.0 {
                    0.08
                } else if d < 5.0 {
                    -0.04
                } else {
                    0.0
                };
                m.velocity = Vec3::new(
                    (p.position[0] - pos[0]) / d * factor,
                    (p.position[1] + 1.0 - pos[1]) / d * 0.06,
                    (p.position[2] - pos[2]) / d * factor,
                );
            } else {
                let t = (m.age as f64 + f64::from(m.id)) * 0.02;
                m.velocity = Vec3::new(t.cos() * 0.03, t.sin() * 0.01, t.sin() * 0.03);
            }
            self.arcadian_fairy_save(*id, data);
        }
        use horizon_client::{
            mobs::Who,
            projectiles::{Projectile, ProjectileKind, arcadian_motion, thrown_type},
        };
        for (id, from, to, kind, bullet, spell) in shots {
            let family = if kind.ends_with(":fairy") {
                ["circle", "pellet"][bullet as usize % 2]
            } else {
                ["circle", "kunai", "pellet"][bullet as usize % 3]
            };
            let Some(base) = thrown_type(&format!("arcadiandream:{family}_bullet")) else {
                continue;
            };
            let t = arcadian_motion::damage_type(
                base,
                (number(&spell, "power", 3.0) * number(config(), "danmakuDamageMultiplier", 1.0))
                    as f32,
            );
            let d = dist(from, to).max(0.01);
            let density = number(&spell, "density", 1.0).clamp(1.0, 40.0) as usize;
            let pattern = spell["pattern"].as_str().unwrap_or("spread");
            let speed = number(&spell, "speed", 0.5);
            let dir = [
                (to[0] - from[0]) / d,
                (to[1] + 0.8 - from[1] - 1.0) / d,
                (to[2] - from[2]) / d,
            ];
            let mut data = spell.clone();
            data["icy"] = json!(kind.ends_with(":ice_fairy"));
            for (n, (angle, layer)) in directions(pattern, density).into_iter().enumerate() {
                let spread = number(&spell, "divergence", 0.0)
                    + if pattern == "spread" || pattern == "rain" {
                        density.saturating_sub(1) as f64
                    } else {
                        0.0
                    };
                let a = angle + ((time as f64 + n as f64 * 13.1).sin() * spread).to_radians();
                let mut p = Projectile::new(
                    ProjectileKind::Thrown(t),
                    Vec3::new(from[0], from[1] + 1.0, from[2]),
                    Some(Who::Mob(id)),
                );
                p.no_gravity = true;
                let s = if pattern == "ray" || pattern == "double" || pattern == "triple" {
                    speed * (1.0 - layer as f64 / density as f64).max(0.1)
                } else {
                    speed
                };
                p.velocity = if pattern == "rain" {
                    Vec3::new(a.sin() * 0.15, speed, a.cos() * 0.15)
                } else {
                    Vec3::new(
                        (dir[0] * a.cos() - dir[2] * a.sin()) * s,
                        dir[1] * s,
                        (dir[0] * a.sin() + dir[2] * a.cos()) * s,
                    )
                };
                self.add_projectile(p);
                if let Some(p) = self.creatures.projectiles.last() {
                    arcadian_motion::register(p.id, data.clone());
                }
            }
        }
        if !self.ui.rules.bool("spawn_mobs")
            || self.world.game_time.rem_euclid(100) != 0
            || time.rem_euclid(24000) > 12000
            || self.ui.difficulty == 0
            || fairy_ids.len() > 24
        {
            return;
        }
        for player in players {
            let angle =
                (time as f64 * 1.618 + player.uuid as u64 as f64).sin() * std::f64::consts::TAU;
            let x = player.position[0] as i32 + (angle.cos() * 32.0) as i32;
            let z = player.position[2] as i32 + (angle.sin() * 32.0) as i32;
            let Ok(h) = self.native_surface_heights(
                dimension,
                &[(x, z)],
                super::native_world_edit::NativeHeightmap::MotionBlockingNoLeaves,
            ) else {
                continue;
            };
            let Some(y) = h.first().copied().flatten() else {
                continue;
            };
            let pos = [x as f64 + 0.5, y as f64, z as f64 + 0.5];
            if config()["fairyCharmOptions"]["canPreventSpawning"]
                .as_bool()
                .unwrap_or(true)
                && charms.iter().any(|c| {
                    c.dimension == dimension
                        && c.charm >= 1
                        && dist(pos, c.position)
                            < number(&config()["fairyCharmOptions"], "distance", 128.0)
                })
            {
                continue;
            }
            if self
                .native_light([x, y, z])
                .0
                .max(self.native_light([x, y, z]).1)
                <= 8
            {
                continue;
            }
            let ground = self.native_block([x, y - 1, z]);
            if !["grass_block", "podzol", "snow_block", "snow", "ice", "mud"]
                .iter()
                .any(|s| ground.ends_with(s))
            {
                continue;
            }
            let biome = self
                .biome_name_at(Vec3::new(x as f64, y as f64, z as f64))
                .unwrap_or_default();
            let snow = self
                .res
                .mob_data()
                .biome_has("minecraft:spawns_snow_foxes", &biome);
            let info = self
                .res
                .biomes()
                .index
                .get(&biome)
                .map(|id| self.res.biomes().get(*id));
            let temperate = self.res.mob_data().biome_has("c:climate_temperate", &biome)
                || self
                    .res
                    .mob_data()
                    .biome_has("c:is_temperate/overworld", &biome)
                || info.is_some_and(|b| {
                    (0.25..=1.0).contains(&b.temperature)
                        && b.downfall >= 0.3
                        && !biome.contains("ocean")
                        && !biome.contains("cave")
                        && !biome.contains("river")
                });
            use horizon_core::random::RandomSource;
            let (name, opt, group) = if snow {
                (
                    "ice_fairy",
                    "spawnIceFairies",
                    1 + self.world.random.next_int_bound(2).unwrap(),
                )
            } else if temperate || biome == "minecraft:sunflower_plains" {
                let weight = if biome == "minecraft:sunflower_plains" {
                    120
                } else {
                    20
                };
                if self.world.random.next_int_bound(80 + weight).unwrap() < 80 {
                    (
                        "fairy",
                        "spawnFairies",
                        1 + self.world.random.next_int_bound(3).unwrap(),
                    )
                } else {
                    ("sunflower_fairy", "spawnSunflowerFairies", 1)
                }
            } else {
                continue;
            };
            if config()["fairyOptions"][opt].as_bool().unwrap_or(true) {
                for member in 0..group {
                    let mut at = pos;
                    at[0] += member as f64 * 0.7;
                    let _ = self.native_spawn_mob(
                        dimension,
                        &format!("arcadiandream:{name}"),
                        at,
                        &NativeSpawn {
                            no_ai: true,
                            no_gravity: true,
                            ..Default::default()
                        },
                    );
                }
            }
        }
    }
}

pub fn directions(pattern: &str, density: usize) -> Vec<(f64, usize)> {
    let n = density.max(1);
    match pattern {
        "ring" => (0..n)
            .map(|i| (i as f64 * std::f64::consts::TAU / n as f64, 0))
            .collect(),
        "arc" => (0..n)
            .map(|i| {
                (
                    (if n == 1 {
                        0.0
                    } else {
                        i as f64 / (n - 1) as f64 - 0.5
                    }) * std::f64::consts::FRAC_PI_4,
                    0,
                )
            })
            .collect(),
        "double" | "triple" => {
            let rays = n.min(if pattern == "double" { 2 } else { 3 });
            (0..rays)
                .flat_map(|r| {
                    (0..(n / rays).max(1)).map(move |i| {
                        (
                            ((r as f64 - (rays - 1) as f64 / 2.0) * 15.0).to_radians(),
                            i,
                        )
                    })
                })
                .collect()
        }
        "ray" => (0..n).map(|i| (0.0, i)).collect(),
        _ => vec![(0.0, 0); n],
    }
}
fn modifier(
    p: &mut horizon_client::motion::PlayerMotion,
    attribute: &str,
    id: &str,
    amount: Option<f64>,
    op: horizon_world::attributes::Operation,
) {
    let Some(key) = p.attributes.key(attribute) else {
        return;
    };
    let Ok(Some(handle)) = p.attributes.get_instance(Some(key)) else {
        return;
    };
    let _ = p.attributes.edit(&handle, |instance, dirty| {
        if let Some(amount) = amount {
            if instance
                .modifier(id)
                .is_some_and(|m| m.amount() == amount && m.operation() == op)
            {
                return Ok(());
            }
            if let Ok(m) = horizon_world::attributes::Modifier::new(id, amount, op) {
                instance.remove(id, dirty)?;
                instance.add_transient(m, dirty)?;
            }
        } else {
            instance.remove(id, dirty)?;
        }
        Ok(())
    });
}
impl Game<'_> {
    fn arcadian_player_effects(&mut self, data: &PlayerData) {
        use horizon_world::attributes::Operation::{AddMultipliedTotal, AddValue};
        if let Some(p) = self.player.as_mut() {
            let n = (data.fatigue_level + 1) as f64;
            let active = data.fatigue > 0;
            modifier(
                p,
                "minecraft:attack_damage",
                "arcadiandream:elixir_fatigue",
                active.then_some(-0.2 * n),
                AddMultipliedTotal,
            );
            modifier(
                p,
                "minecraft:movement_speed",
                "arcadiandream:elixir_fatigue",
                active.then_some(-0.1 * n),
                AddMultipliedTotal,
            );
            modifier(
                p,
                "minecraft:max_health",
                "arcadiandream:elixir_fatigue",
                active.then_some(-2.0 * n),
                AddValue,
            );
            modifier(
                p,
                "minecraft:scale",
                "arcadiandream:miracle_mallet",
                data.tiny.then_some(-0.75),
                AddMultipliedTotal,
            );
        }
        if data.elixir >= 2 {
            self.potions.effects.list.retain(|e| !harmful(e.id));
        }
    }
}

fn client_player() -> &'static std::sync::Mutex<PlayerData> {
    static P: OnceLock<std::sync::Mutex<PlayerData>> = OnceLock::new();
    P.get_or_init(Default::default)
}
pub fn save_now() {
    STATE.with(|s| save(&s.borrow()));
}
impl Game<'_> {
    pub(super) fn arcadian_read_uuid(&self) -> Option<[i32; 4]> {
        if self.remote.is_some() {
            return client_player().lock().unwrap().uuid;
        }
        self.lan_acting_uuid
            .map(horizon_client::lan::playerdata::uuid_ints)
            .or(self.level.player_uuid)
    }
    pub(super) fn arcadian_can_punch(&self) -> bool {
        if !self.native_on("arcadiandream") {
            return false;
        }
        let equipped = if self.remote.is_some() {
            client_player()
                .lock()
                .unwrap()
                .accessories
                .contains_key("necklace")
        } else {
            self.arcadian_read_uuid()
                .map(horizon_client::lan::playerdata::ints_uuid)
                .is_some_and(|id| {
                    state(self.native_world_dir(), |s| {
                        s.players
                            .get(&uuid(id))
                            .is_some_and(|p| p.accessories.contains_key("necklace"))
                    })
                })
        };
        equipped
            && !self
                .cooldowns
                .is_on_cooldown("arcadiandream:magatama_necklace")
    }
    pub(super) fn arcadian_punch(&mut self, index: usize) -> bool {
        if !self.creatures.projectiles.get(index).is_some_and(|p| {
            p.kind.id().starts_with("arcadiandream:") && p.kind.id().ends_with("_bullet")
        }) {
            return false;
        }
        let own = self.acting_player_uuid();
        if self.arcadian_can_punch() && self.creatures.projectiles[index].owner_uuid != Some(own) {
            let p = self.creatures.projectiles.remove(index);
            horizon_client::projectiles::arcadian_motion::forget(p.id);
            self.add_cooldown("arcadiandream:magatama_necklace".into(), 10);
            self.arcadian_cancelled(1, false);
            self.reset_attack_strength();
            self.swing_arm();
        }
        true
    }
}

pub fn harmful(id: &str) -> bool {
    id != "arcadiandream:elixir_fatigue"
        && horizon_client::brewing::effect(id)
            .is_some_and(|e| e.category == horizon_client::brewing::Category::Harmful)
}
impl Game<'_> {
    pub(super) fn arcadian_effect_immune(&mut self, id: &str) -> bool {
        if !self.native_on("arcadiandream") || !harmful(id) {
            return false;
        }
        let key = uuid(self.native_acting_id());
        state(self.native_world_dir(), |s| {
            s.players.get(&key).is_some_and(|p| p.elixir >= 2)
        })
    }
}

pub fn reconnect(id: u128) {
    STATE.with(|s| {
        s.borrow_mut().sent.remove(&id);
    });
}

impl Game<'_> {
    pub(super) fn arcadian_own_shot(&self, p: &horizon_client::projectiles::Projectile) -> bool {
        let Some(own) = self.arcadian_read_uuid() else {
            return false;
        };
        if p.owner_uuid == Some(own) {
            return true;
        }
        let data = horizon_client::projectiles::arcadian_motion::data(p.id);
        serde_json::from_value::<[i32; 4]>(data["owner"].clone()).ok() == Some(own)
    }
}
