// SPDX-License-Identifier: EUPL-1.2-or-later
// Original drop rates: Maxmani and contributors' Arcadian Dream 0.1.0.
use serde_json::{Value, json};
thread_local! {static ENABLED: std::cell::Cell<bool> = const {std::cell::Cell::new(false)};}
pub fn root(root: &std::path::Path) {
    ENABLED.with(|v| v.set(root.join("data/arcadiandream/loot_table").is_dir()));
}
pub fn extend(id: &str, table: &mut Value) {
    if !ENABLED.with(|v| v.get()) {
        return;
    }
    let mut added = Vec::new();
    let item = |name: &str| json!({"type":"minecraft:item","name":format!("arcadiandream:{name}")});
    let pool = |chance: f64, entry: Value| json!({"rolls":1,"conditions":[{"condition":"minecraft:random_chance","chance":chance}],"entries":[entry]});
    let path = id.strip_prefix("minecraft:chests/").unwrap_or("");
    if [
        "simple_dungeon",
        "end_city_treasure",
        "bastion_treasure",
        "desert_pyramid",
        "woodland_mansion",
        "ancient_city_ice_box",
        "ancient_city",
        "buried_treasure",
        "jungle_temple",
        "abandoned_mineshaft",
        "stronghold_crossing",
        "stronghold_corridor",
    ]
    .contains(&path)
        || id == "arcadiandream:chests/abandoned_shrine_treasure"
    {
        let mut p = pool(0.05, item("time_orb"));
        p["entries"] = json!([item("time_orb"), item("folding_chair")]);
        added.push(p);
    }
    if path == "end_city_treasure" {
        for (name, chance) in [
            ("dragon_gem", 0.25),
            ("hihiirokane_upgrade_smithing_template", 0.1),
        ] {
            let mut p = pool(chance, item(name));
            p["functions"] = json!([{"function":"minecraft:set_count","count":{"type":"minecraft:uniform","min":1.0,"max":2.0}}]);
            added.push(p);
        }
    }
    let pattern = match path {
        "shipwreck_supply" | "shipwreck_treasure" | "shipwreck_map" => {
            Some(("spread_pattern_template", 0.1))
        }
        "pillager_outpost" => Some(("ray_pattern_template", 0.25)),
        "nether_bridge" => Some(("ring_pattern_template", 0.25)),
        "ancient_city" => Some(("arc_pattern_template", 0.25)),
        "abandoned_mineshaft" => Some(("double_pattern_template", 0.25)),
        "stronghold_corridor" | "stronghold_crossing" | "stronghold_library" => {
            Some(("triple_pattern_template", 0.1))
        }
        _ => None,
    };
    if let Some((name, chance)) = pattern {
        added.push(pool(chance, item(name)));
    }
    if ["igloo_chest", "ancient_city_ice_box"].contains(&path) {
        added.push(pool(0.1, item("enchanted_ice")));
    }
    if path.starts_with("shipwreck_") {
        let mut p = pool(0.1, item("enchanted_ice"));
        p["conditions"].as_array_mut().unwrap().push(json!({"condition":"minecraft:any_of","terms":[{"condition":"minecraft:location_check","predicate":{"biomes":"minecraft:frozen_ocean"}},{"condition":"minecraft:location_check","predicate":{"biomes":"minecraft:deep_frozen_ocean"}}]}));
        added.push(p);
    }
    if id == "minecraft:gameplay/fishing/fish" || id == "minecraft:gameplay/fishing/treasure" {
        if let Some(pools) = table["pools"].as_array_mut() {
            for p in pools {
                if let Some(entries) = p["entries"].as_array_mut() {
                    let mut e = item(if id.ends_with("/fish") {
                        "lamprey"
                    } else {
                        "life_fragment"
                    });
                    if id.ends_with("/fish") {
                        e["weight"] = json!(12);
                    }
                    entries.push(e);
                }
            }
        }
    }
    for p in &mut added{current(p);}
    if !added.is_empty() {
        if !table["pools"].is_array() {
            table["pools"] = json!([]);
        }
        table["pools"].as_array_mut().unwrap().extend(added);
    }
}

fn current(v:&mut Value){
 match v{
  Value::Array(a)=>for v in a{current(v)},
  Value::Object(o)=>{for v in o.values_mut(){current(v)}
   if let Some(v)=o.remove("function"){o.insert("type".into(),v);}
   if !o.contains_key("type"){if let Some(v)=o.remove("condition"){o.insert("type".into(),v);}}
   if let Some(v)=o.remove("functions"){o.insert("modifier".into(),v);}
  },_=>{}
 }
}
