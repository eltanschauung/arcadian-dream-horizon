// Native health uses the decoded catalog, not datapack damage_type JSON.
fn hook(v: &serde_json::Value, resources: &serde_json::Value) -> core::ops::ControlFlow<io::Result<Catalog>> {
    if v["types"].as_array().is_none_or(|types| types.iter().any(|t| t["id"] == "arcadiandream:danmaku")) {
        return core::ops::ControlFlow::Continue(());
    }
    let mut v = v.clone();
    let mut tags = resources.clone();
    for name in ["danmaku", "danmaku_sharp"] {
        v["types"].as_array_mut().unwrap().push(serde_json::json!({
            "id": format!("arcadiandream:{name}"), "message_id": if name == "danmaku_sharp" { "danmakuSharp" } else { "danmaku" },
            "exhaustion": "3dcccccd", "scaling": "never", "effects": "hurt", "death_message_type": "default"
        }));
    }
    for (tag, values) in [
        ("minecraft:is_projectile", vec!["arcadiandream:danmaku", "arcadiandream:danmaku_sharp"]),
        ("minecraft:bypasses_armor", vec!["arcadiandream:danmaku_sharp"]),
        ("arcadiandream:is_danmaku", vec!["arcadiandream:danmaku", "arcadiandream:danmaku_sharp"]),
    ] {
        let mut data = tags[tag].as_str().and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok()).unwrap_or(serde_json::json!({"values":[]}));
        data["values"].as_array_mut().unwrap().extend(values.into_iter().map(|s|serde_json::json!(s)));
        tags[tag] = serde_json::Value::String(data.to_string());
    }
    // The appended ID above makes the recursive parse pass through unmodified.
    core::ops::ControlFlow::Break(Catalog::from_json(&v, &tags))
}

