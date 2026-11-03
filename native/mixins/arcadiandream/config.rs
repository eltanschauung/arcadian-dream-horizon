// SPDX-License-Identifier: EUPL-1.2-or-later
use serde_json::{Value, json};
use std::{path::PathBuf, sync::OnceLock};
pub fn config() -> &'static Value {
    static CONFIG: OnceLock<Value> = OnceLock::new();
    CONFIG.get_or_init(||{
        let root=std::env::var_os("HORIZON_HOME").map(PathBuf::from).unwrap_or_else(||PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".local/share/horizon"));
        let path=root.join("config/arcadiandream.json");let defaults=json!({
            "chiselOptions":{"canUse":true,"maxDistance":16},"nueTridentOptions":{"canHaveImpaling":true},
            "hisouSwordOptions":{"minHeightForPeaches":196,"canDisableShields":true},"mochiMalletOptions":{"lowViolence":false},
            "houraiElixirOptions":{"canDrink":false,"nerfElixir":true,"allowDying":false,"elixirFatigueThreshold":1},
            "foldingChairOptions":{"knockbackStrength":1.5},"icicleSwordOptions":{"meltsInstantly":true},
            "fairyCharmOptions":{"canPreventSpawning":true,"distance":128},"danmakuCooldownMultiplier":1,"danmakuDamageMultiplier":1,
            "cooldownPerBulletType":true,"fairyOptions":{"spawnFairies":true,"spawnSunflowerFairies":true,"spawnIceFairies":true}});
        let mut result=defaults.clone();if let Ok(b)=std::fs::read(&path) {match serde_json::from_slice::<Value>(&b){Ok(v)=>merge(&mut result,v),Err(e)=>eprintln!("Arcadian Dream: invalid config {e}; using defaults")}}
        else {let _=std::fs::create_dir_all(path.parent().unwrap());let _=std::fs::write(path,serde_json::to_vec_pretty(&defaults).unwrap());}result
    })
}
fn merge(a: &mut Value, b: Value) {
    if let (Some(a), Some(b)) = (a.as_object_mut(), b.as_object()) {
        for (k, v) in b {
            if let Some(old) = a.get_mut(k) {
                if old.is_object() {
                    merge(old, v.clone())
                } else {
                    *old = v.clone()
                }
            }
        }
    }
}
