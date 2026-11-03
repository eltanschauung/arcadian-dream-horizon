// SPDX-License-Identifier: EUPL-1.2-or-later
use super::Game;
use horizon_core::random::RandomSource;
use horizon_world::{
    attachments::{AttachmentCodec, AttachmentError, AttachmentType, Holder},
    nbt::{Compound, Tag},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};
#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Fairy {
    pub initialized: bool,
    pub variant: u8,
    pub attack: u8,
    pub bullet: u8,
    pub color: u32,
    pub offset: i32,
    pub random: i32,
    pub anger: u32,
    pub cooldown: i32,
}
static FAIRY: AttachmentType<Fairy> = AttachmentType {
    namespace: "arcadiandream",
    name: "fairy",
    default: Fairy::default,
    copy_on_death: false,
    codec: Some(AttachmentCodec {
        encode: |v| {
            Ok(Compound::from([(
                "json".into(),
                Tag::String(
                    serde_json::to_string(v)
                        .map_err(|_| AttachmentError::Encode)?
                        .as_str()
                        .into(),
                ),
            )]))
        },
        decode: |c| {
            if let Some(Tag::String(s)) = c.get(&"json".into()) {
                serde_json::from_str(&s.to_string_lossy()).map_err(|_| AttachmentError::Decode)
            } else {
                Err(AttachmentError::Decode)
            }
        },
    }),
};
fn visuals() -> &'static Mutex<HashMap<u32, Fairy>> {
    static M: OnceLock<Mutex<HashMap<u32, Fairy>>> = OnceLock::new();
    M.get_or_init(Default::default)
}
pub fn variant(id: u32) -> u8 {
    visuals().lock().unwrap().get(&id).map_or(0, |f| f.variant)
}
pub fn received(rows: &Value) {
    if let Some(rows) = rows.as_array() {
        let mut v = visuals().lock().unwrap();
        if v.len() > 8192 {
            v.clear()
        }
        for row in rows {
            if let (Some(id), Ok(f)) = (
                row[0].as_u64(),
                serde_json::from_value::<Fairy>(row[1].clone()),
            ) {
                v.insert(id as u32, f);
            }
        }
    }
}
impl Game<'_> {
    pub(super) fn arcadian_fairy_data(&mut self, id: u32, kind: &str) -> Fairy {
        let mut data = self
            .native_attachments(Holder::Entity(id))
            .and_then(|a| a.get(&FAIRY))
            .unwrap_or_default();
        if !data.initialized {
            data.initialized = true;
            data.variant = self.world.random.next_int_bound(4).unwrap() as u8;
            data.attack = self
                .world
                .random
                .next_int_bound(if kind.ends_with(":fairy") { 6 } else { 8 })
                .unwrap() as u8;
            data.bullet = self
                .world
                .random
                .next_int_bound(if kind.ends_with(":fairy") { 2 } else { 3 })
                .unwrap() as u8;
            data.offset = self.world.random.next_int_bound(11).unwrap() - 5;
            data.random = self.world.random.next_int_bound(11).unwrap() + 1;
            data.color = [
                0xffffff, 0x808080, 0x404040, 0x0a0a0a, 0x964b00, 0xff0000, 0xffa500, 0xffe400,
                0xbfff00, 0x00ff00, 0x00ffff, 0x3ab3da, 0x1d1dff, 0xa023f0, 0xff00ff, 0xf38baa,
            ][self.world.random.next_int_bound(16).unwrap() as usize];
        }
        data
    }
    pub(super) fn arcadian_fairy_save(&mut self, id: u32, data: Fairy) {
        visuals().lock().unwrap().insert(id, data.clone());
        let _ = self
            .native_attachments(Holder::Entity(id))
            .and_then(|a| a.set(&FAIRY, data));
    }
}
pub fn attack(kind: &str, data: &Fairy) -> Value {
    let name = kind.trim_start_matches("arcadiandream:");
    let row = &super::arcadian::definitions()["fairy_attacks"][name][data.attack as usize];
    let mut out = serde_json::json!({"pattern":row["pattern"],"cooldown":row["cooldown"].as_i64().unwrap_or(30)+data.offset as i64,"color":if name=="ice_fairy"{0x00ffff}else{data.color}});
    if let Some(args) = row["args"].as_array() {
        for (key, v) in [
            "density",
            "speed",
            "divergence",
            "power",
            "duration",
            "color",
            "gravity",
        ]
        .into_iter()
        .zip(args)
        {
            if key == "color" {
                continue;
            }
            let s = v.as_str().unwrap_or("0");
            let n = if s.starts_with("getRandomNumberProvider()") {
                data.random as f64
                    + s.split('+')
                        .nth(1)
                        .and_then(|s| s.trim().parse::<f64>().ok())
                        .unwrap_or(0.0)
            } else {
                s.parse().unwrap_or(0.0)
            };
            out[key] = serde_json::json!(n);
        }
    }
    out
}
