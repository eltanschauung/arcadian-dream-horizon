// SPDX-License-Identifier: EUPL-1.2-or-later
use serde_json::{Value, json};
use std::path::PathBuf;
fn path() -> PathBuf {
    let home = std::env::var_os("HORIZON_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".local/share/horizon")
        });
    home.join("config/arcadiandream.json")
}
fn read() -> Value {
    std::fs::read(path())
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_else(|| super::arcadian::config().clone())
}
fn rows(doc: &Value) -> Vec<(Vec<String>, Value)> {
    let mut r = Vec::new();
    if let Some(map) = doc.as_object() {
        for (k, v) in map {
            if let Some(m) = v.as_object() {
                for (n, v) in m {
                    r.push((vec![k.clone(), n.clone()], v.clone()));
                }
            } else {
                r.push((vec![k.clone()], v.clone()));
            }
        }
    }
    r
}
fn label(keys: &[String]) -> String {
    keys.iter()
        .map(|k| {
            let mut s = String::new();
            for c in k.chars() {
                if c.is_uppercase() {
                    s.push(' ')
                }
                s.push(c);
            }
            s
        })
        .collect::<Vec<_>>()
        .join(": ")
}
pub fn buttons() -> Vec<String> {
    let doc = read();
    let mut out = vec!["Changes apply after restarting the game/server".into()];
    out.extend(rows(&doc).into_iter().map(|(k, v)| {
        format!(
            "{}: {}",
            label(&k),
            if let Some(on) = v.as_bool() {
                if on { "ON".into() } else { "OFF".into() }
            } else {
                v.to_string()
            }
        )
    }));
    out
}
pub fn click(index: usize) {
    if index == 0 {
        return;
    }
    let mut doc = read();
    let list = rows(&doc);
    let Some((keys, value)) = list.get(index - 1) else {
        return;
    };
    let mut target = &mut doc;
    for key in keys {
        target = &mut target[key];
    }
    *target = if let Some(on) = value.as_bool() {
        json!(!on)
    } else {
        let name = keys.last().unwrap().as_str();
        let (step, max, min) = match name {
            "maxDistance" => (1.0, 64.0, 1.0),
            "minHeightForPeaches" => (8.0, 256.0, 64.0),
            "distance" => (8.0, 128.0, 25.0),
            "knockbackStrength" => (0.5, 10.0, 0.0),
            "elixirFatigueThreshold" => (1.0, 5.0, 1.0),
            _ => (1.0, 4.0, 1.0),
        };
        let n = value.as_f64().unwrap_or(min) + step;
        let n = if n > max { min } else { n };
        if value.is_i64() || value.is_u64() {
            json!(n as i64)
        } else {
            json!(n)
        }
    };
    let path = path();
    let save = (|| -> std::io::Result<()> {
        std::fs::create_dir_all(path.parent().unwrap())?;
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, serde_json::to_vec_pretty(&doc)?)?;
        std::fs::rename(tmp, path)
    })();
    if let Err(e) = save {
        eprintln!("Arcadian Dream settings: {e}");
    }
}
