// SPDX-License-Identifier: EUPL-1.2-or-later
use super::{Hit, Projectile, ProjectileKind, ProjectileWorld, ThrownType};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};
fn shots() -> &'static Mutex<HashMap<u32, Value>> {
    static S: OnceLock<Mutex<HashMap<u32, Value>>> = OnceLock::new();
    S.get_or_init(Default::default)
}
pub fn register(id: u32, data: Value) {
    if id != 0 {
        let mut s = shots().lock().unwrap();
        if s.len() > 8192 {
            if let Some(old) = s.keys().copied().min() {
                s.remove(&old);
            }
        }
        s.insert(id, data);
    }
}
pub fn clear() {
    shots().lock().unwrap().clear();
    pending().lock().unwrap().clear();
}
pub fn forget(id: u32) {
    shots().lock().unwrap().remove(&id);
}
pub fn data(id: u32) -> Value {
    shots()
        .lock()
        .unwrap()
        .get(&id)
        .cloned()
        .unwrap_or_else(|| json!({}))
}
pub fn enlarge(id: u32) {
    let mut s = shots().lock().unwrap();
    if let Some(v) = s.get_mut(&id) {
        v["scale"] = json!((v["scale"].as_f64().unwrap_or(1.0) + 0.75).min(4.0));
    }
}
pub fn damage_type(base: &'static ThrownType, power: f32) -> &'static ThrownType {
    static TYPES: OnceLock<Mutex<HashMap<(&'static str, u32), &'static ThrownType>>> =
        OnceLock::new();
    // Quantizing to 1/100 HP bounds the cache even with arbitrary shot modifiers.
    let power = (power.clamp(0.1, 60.0) * 100.0).round() as u32;
    *TYPES
        .get_or_init(Default::default)
        .lock()
        .unwrap()
        .entry((base.id, power))
        .or_insert_with(|| {
            Box::leak(Box::new(ThrownType {
                id: base.id,
                item: base.item,
                damage: power as f32 / 100.0,
            }))
        })
}
pub fn tick(p: &mut Projectile, w: &impl ProjectileWorld, impact: bool) -> Option<(bool, Hit)> {
    if !matches!(p.kind,ProjectileKind::Thrown(t) if t.id.starts_with("arcadiandream:")&&t.id.ends_with("_bullet"))
    {
        return None;
    }
    prepare(p);
    let d = data(p.id);
    let duration = d["duration"].as_f64().unwrap_or(100.0).clamp(1.0, 1200.0) as u64;
    if p.age as u64 >= duration {
        shots().lock().unwrap().remove(&p.id);
        return Some((false, Hit::None));
    }
    p.previous = p.position;
    p.previous_yaw = p.yaw;
    p.previous_pitch = p.pitch;
    p.age += 1;
    let gravity = d["gravity"].as_f64().unwrap_or(0.0);
    p.velocity.y -= gravity;
    let from = p.position;
    let to = horizon_core::geometry::Vec3::new(
        from.x + p.velocity.x,
        from.y + p.velocity.y,
        from.z + p.velocity.z,
    );
    let block = w.clip(from, to);
    let end = block.map_or(to, |b| b.0);
    let mut best = None;
    for (who, bounds) in w.targets() {
        if Some(*who) == p.owner
            || d["hit_entities"].as_array().is_some_and(|ids| {
                ids.iter()
                    .any(|id| id.as_str() == Some(&format!("{who:?}")))
            })
        {
            continue;
        }
        let radius = match p.kind.id() {
            "arcadiandream:bubble_bullet" => 0.375,
            "arcadiandream:star_bullet" => 0.25,
            "arcadiandream:pellet_bullet" => 0.075,
            _ => 0.125,
        } * d["scale"].as_f64().unwrap_or(1.0);
        let b = bounds.inflate(horizon_core::geometry::Vec3::new(radius, radius, radius));
        if let Some(at) = b.clip(from, end) {
            let distance =
                (at.x - from.x).powi(2) + (at.y - from.y).powi(2) + (at.z - from.z).powi(2);
            if best
                .as_ref()
                .is_none_or(|(_, _, old): &(_, _, f64)| distance < *old)
            {
                best = Some((*who, at, distance));
            }
        }
    }
    let hit = if impact {
        if let Some((who, at, _)) = best {
            Hit::Entity { who, at }
        } else if let Some((at, pos, face)) = block {
            Hit::Block { pos, at, face }
        } else {
            Hit::None
        }
    } else {
        Hit::None
    };
    p.position = match hit {
        Hit::Entity { at, .. } | Hit::Block { at, .. } => at,
        Hit::None => to,
    };
    p.left_owner = true;
    let horizontal = (p.velocity.x.powi(2) + p.velocity.z.powi(2)).sqrt();
    p.yaw = p.velocity.x.atan2(p.velocity.z).to_degrees() as f32;
    p.pitch = p.velocity.y.atan2(horizontal).to_degrees() as f32;
    let piercing = p.kind.id() == "arcadiandream:kunai_bullet" && matches!(hit, Hit::Entity { .. });
    if let Hit::Entity { who, .. } = hit {
        if piercing {
            let mut s = shots().lock().unwrap();
            let v = s.entry(p.id).or_insert_with(|| json!({}));
            if !v["hit_entities"].is_array() {
                v["hit_entities"] = json!([]);
            }
            v["hit_entities"]
                .as_array_mut()
                .unwrap()
                .push(json!(format!("{who:?}")));
        }
    }
    Some((matches!(hit, Hit::None) || piercing, hit))
}
// Saved throwables are assigned a new entity id after decoding. Carry their shot
// data by the immutable initial position/velocity until the first tick assigns it.
type SpawnKey = [u64; 6];
fn key(p: &Projectile) -> SpawnKey {
    [
        p.position.x.to_bits(),
        p.position.y.to_bits(),
        p.position.z.to_bits(),
        p.velocity.x.to_bits(),
        p.velocity.y.to_bits(),
        p.velocity.z.to_bits(),
    ]
}
fn pending() -> &'static Mutex<HashMap<SpawnKey, Vec<(Value, u32)>>> {
    static M: OnceLock<Mutex<HashMap<SpawnKey, Vec<(Value, u32)>>>> = OnceLock::new();
    M.get_or_init(Default::default)
}
pub fn restore(p: &mut Projectile, v: Value, age: u32) {
    p.age = age;
    pending()
        .lock()
        .unwrap()
        .entry(key(p))
        .or_default()
        .push((v, age));
}
pub fn prepare(p: &mut Projectile) {
    if shots().lock().unwrap().contains_key(&p.id) {
        return;
    }
    let v = pending()
        .lock()
        .unwrap()
        .get_mut(&key(p))
        .and_then(Vec::pop);
    if let Some((v, age)) = v {
        p.age = age;
        if let ProjectileKind::Thrown(t) = p.kind {
            p.kind =
                ProjectileKind::Thrown(damage_type(t, v["power"].as_f64().unwrap_or(2.0) as f32));
        }
        register(p.id, v);
    }
}
pub fn drawn(p: &Projectile) -> Option<crate::items::ItemStack> {
    if let ProjectileKind::Thrown(t) = p.kind {
        if t.id.starts_with("arcadiandream:") && t.id.ends_with("_bullet") {
            let mut s = crate::items::ItemStack::new(t.item, 1);
            let data = data(p.id);
            s.data_mut().dyed_color = Some(data["color"].as_i64().unwrap_or(0xff0000) as i32);
            let scale = ((data["scale"].as_f64().unwrap_or(1.0) - 1.0) / 0.75)
                .round()
                .clamp(0.0, 4.0) as u8;
            let family =
                t.id.trim_start_matches("arcadiandream:")
                    .trim_end_matches("_bullet");
            let phase = if family == "star" { p.age % 16 } else { 0 };
            s.data_mut().set_loot_component(
                "minecraft:item_model",
                &json!(format!(
                    "arcadiandream:arcadian_projectile/{family}_{scale}_{phase}"
                )),
            );
            return Some(s);
        }
    }
    None
}
