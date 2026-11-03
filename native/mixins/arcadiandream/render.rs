// SPDX-License-Identifier: EUPL-1.2-or-later
// Geometry and keyframes originate in Arcadian Dream, CC-BY-SA-4.0.
use super::{
    Game,
    native_block_entity_draw::{BlockEntityDraw, CubeFace},
    native_mob_draw::NativeMobRender,
};
use serde_json::Value;
fn vector(v: &Value, default: [f32; 3]) -> [f32; 3] {
    if let Some(a) = v.as_array() {
        std::array::from_fn(|i| {
            a.get(i)
                .and_then(Value::as_f64)
                .unwrap_or(default[i] as f64) as f32
        })
    } else {
        default
    }
}
fn frame(v: &Value, time: f32, default: [f32; 3]) -> [f32; 3] {
    if v.is_array() {
        return vector(v, default);
    }
    if v["vector"].is_array() {
        return vector(&v["vector"], default);
    }
    let Some(keys) = v.as_object() else {
        return default;
    };
    let mut frames = keys
        .iter()
        .filter_map(|(s, v)| s.parse::<f32>().ok().map(|t| (t, v)))
        .collect::<Vec<_>>();
    frames.sort_by(|a, b| a.0.total_cmp(&b.0));
    if frames.is_empty() {
        return default;
    }
    let before = frames
        .iter()
        .rev()
        .find(|(t, _)| *t <= time)
        .unwrap_or(&frames[0]);
    let after = frames
        .iter()
        .find(|(t, _)| *t >= time)
        .unwrap_or(frames.last().unwrap());
    let a = vector(
        before
            .1
            .get("post")
            .or_else(|| before.1.get("vector"))
            .unwrap_or(before.1),
        default,
    );
    let b = vector(
        after
            .1
            .get("pre")
            .or_else(|| after.1.get("vector"))
            .unwrap_or(after.1),
        default,
    );
    let mut t = if after.0 == before.0 {
        0.0
    } else {
        (time - before.0) / (after.0 - before.0)
    };
    match after.1["easing"].as_str().unwrap_or("linear") {
        "easeInOutQuad" => {
            t = if t < 0.5 {
                2.0 * t * t
            } else {
                1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
            }
        }
        "easeInCirc" => t = 1.0 - (1.0 - t * t).max(0.0).sqrt(),
        _ => {}
    }
    std::array::from_fn(|i| a[i] + (b[i] - a[i]) * t)
}
impl Game<'_> {
    pub(super) fn arcadian_draw_fairy(
        &self,
        m: &NativeMobRender<'_>,
        draw: &mut BlockEntityDraw,
    ) -> bool {
        let Some(name) = m.kind.strip_prefix("arcadiandream:") else {
            return false;
        };
        if !["fairy", "sunflower_fairy", "ice_fairy"].contains(&name) {
            return false;
        }
        let data = super::arcadian::definitions();
        let model = &data["models"][format!("geo/entity/{name}.geo.json")]["minecraft:geometry"][0];
        let Some(bones) = model["bones"].as_array() else {
            return false;
        };
        let animname = if name == "sunflower_fairy" {
            name
        } else {
            "fairy"
        };
        let animations =
            &data["models"][format!("animations/entity/{animname}.animation.json")]["animations"];
        let anim = &animations[if m.walk_speed > 0.02 {
            "move"
        } else if m.on_ground {
            "idle.ground"
        } else {
            "idle"
        }];
        let time = (m.age / 20.0) % anim["animation_length"].as_f64().unwrap_or(4.0) as f32;
        let skin = if name == "fairy" {
            format!(
                "fairy_{}",
                ["blue", "red", "green", "yellow"][super::arcadian_fairies::variant(m.id) as usize]
            )
        } else {
            name.to_string()
        };
        let sprite = format!("arcadiandream:entity/fairy/{skin}");
        let wing_sprite = format!("arcadiandream:entity/fairy/{name}_wings");
        let tw = model["description"]["texture_width"]
            .as_f64()
            .unwrap_or(64.0) as f32;
        let th = model["description"]["texture_height"]
            .as_f64()
            .unwrap_or(64.0) as f32;

        draw.push();
        draw.rotate_about([0.0, 1.0, 0.0], 180.0, [0.0; 3]);
        for (i, b) in bones.iter().enumerate() {
            if b["parent"].is_null() {
                bone(
                    self,
                    draw,
                    bones,
                    i,
                    anim,
                    time,
                    m,
                    &sprite,
                    &wing_sprite,
                    tw,
                    th,
                );
            }
        }
        draw.pop();
        true
    }
}

fn bone(
    g: &Game<'_>,
    draw: &mut BlockEntityDraw,
    bones: &[Value],
    index: usize,
    anim: &Value,
    time: f32,
    m: &NativeMobRender<'_>,
    sprite: &str,
    wings: &str,
    tw: f32,
    th: f32,
) {
    let b = &bones[index];
    let name = b["name"].as_str().unwrap_or("");
    let a = &anim["bones"][name];
    draw.push();
    let translation = frame(&a["position"], time, [0.0; 3]);
    draw.translate(translation.map(|v| v / 16.0));
    let pivot = vector(&b["pivot"], [0.0; 3]).map(|v| v / 16.0);
    let base = vector(&b["rotation"], [0.0; 3]);
    let mut rotation = frame(&a["rotation"], time, [0.0; 3]);
    if name == "head" || name == "bipedHead" {
        rotation[0] += m.pitch;
        rotation[1] += m.head_yaw;
    }
    for i in [2, 1, 0] {
        let mut axis = [0.0; 3];
        axis[i] = 1.0;
        draw.rotate_about(axis, base[i] + rotation[i], pivot);
    }
    draw.scale(frame(&a["scale"], time, [1.0; 3]));
    if let Some(cubes) = b["cubes"].as_array() {
        for c in cubes {
            draw.push();
            let origin = vector(&c["origin"], [0.0; 3]);
            let size = vector(&c["size"], [0.0; 3]);
            let inflate = c["inflate"].as_f64().unwrap_or(0.0) as f32;
            let from = std::array::from_fn(|i| origin[i] - inflate);
            let to = std::array::from_fn(|i| origin[i] + size[i] + inflate);
            let cp = vector(&c["pivot"], [0.0; 3]).map(|v| v / 16.0);
            let r = vector(&c["rotation"], [0.0; 3]);
            for i in [2, 1, 0] {
                let mut axis = [0.0; 3];
                axis[i] = 1.0;
                draw.rotate_about(axis, r[i], cp);
            }
            let s = if name.to_ascii_lowercase().contains("wing") {
                wings
            } else {
                sprite
            };
            let faces = std::array::from_fn(|i| {
                let face = ["down", "up", "north", "south", "west", "east"][i];
                let v = &c["uv"][face];
                let uv = if v.is_object() {
                    let u = vector(&v["uv"], [0.0; 3]);
                    let n = vector(&v["uv_size"], [0.0; 3]);
                    [
                        u[0] * 16.0 / tw,
                        u[1] * 16.0 / th,
                        (u[0] + n[0]) * 16.0 / tw,
                        (u[1] + n[1]) * 16.0 / th,
                    ]
                } else if let Some(a) = c["uv"].as_array() {
                    let u = a[0].as_f64().unwrap_or(0.0) as f32;
                    let v = a[1].as_f64().unwrap_or(0.0) as f32;
                    let [x, y, z] = size;
                    let (du, dv, w, h) = match face {
                        "down" => (z + x, 0.0, x, z),
                        "up" => (z, 0.0, x, z),
                        "north" => (z, z, x, y),
                        "south" => (z + x + z, z, x, y),
                        "west" => (z + x, z, z, y),
                        _ => (0.0, z, z, y),
                    };
                    [
                        (u + du) * 16.0 / tw,
                        (v + dv) * 16.0 / th,
                        (u + du + w) * 16.0 / tw,
                        (v + dv + h) * 16.0 / th,
                    ]
                } else {
                    return None;
                };
                Some(CubeFace {
                    sprite: s,
                    uv: Some(uv),
                })
            });
            g.native_draw_cube(
                draw,
                from,
                to,
                faces,
                if m.hurt_time > 0 { 0xff8888 } else { 0xffffff },
            );
            draw.pop();
        }
    }
    for (j, child) in bones.iter().enumerate() {
        if child["parent"].as_str() == Some(name) {
            bone(g, draw, bones, j, anim, time, m, sprite, wings, tw, th);
        }
    }
    draw.pop();
}

impl Game<'_> {
    pub(super) fn arcadian_draw_hat(
        &self,
        m: &NativeMobRender<'_>,
        local: bool,
        draw: &mut BlockEntityDraw,
    ) {
        let wearing = if local {
            self.armor[0]
                .as_ref()
                .is_some_and(|s| s.item.as_ref() == "arcadiandream:ordinary_hat")
        } else {
            self.mobs
                .iter()
                .find(|p| p.id == m.id)
                .is_some_and(|p| p.extra.head.as_deref() == Some("arcadiandream:ordinary_hat"))
        };
        if !wearing {
            return;
        }
        let model = &super::arcadian::definitions()["models"]["geo/armor/ordinary_armor.geo.json"]
            ["minecraft:geometry"][0];
        let Some(bones) = model["bones"].as_array() else {
            return;
        };
        draw.push();
        draw.rotate_about([0.0, 1.0, 0.0], 180.0, [0.0; 3]);
        for (i, b) in bones.iter().enumerate() {
            if b["parent"].is_null() {
                bone(
                    self,
                    draw,
                    bones,
                    i,
                    &Value::Null,
                    m.age / 20.0,
                    m,
                    "arcadiandream:models/armor/ordinary_armor",
                    "arcadiandream:models/armor/ordinary_armor",
                    64.0,
                    64.0,
                );
            }
        }
        draw.pop();
    }
}
