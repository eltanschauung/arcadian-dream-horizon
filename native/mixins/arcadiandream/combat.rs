// SPDX-License-Identifier: EUPL-1.2-or-later
use super::Game;
use super::arcadian::{custom, number};
use horizon_client::{
    items::ItemStack,
    mobs::{Damage, DamageKind, Who},
    projectiles::{Hit, Projectile, ProjectileKind, arcadian_motion},
};
impl Game<'_> {
    pub(super) fn arcadian_projectile_hit(&mut self, p: &Projectile, hit: Hit) -> bool {
        if !self.native_on("arcadiandream") {
            return false;
        }
        let ProjectileKind::Thrown(t) = p.kind else {
            return false;
        };
        if !t.id.starts_with("arcadiandream:") || !t.id.ends_with("_bullet") {
            return false;
        }
        let Hit::Entity { who, .. } = hit else {
            return true;
        };
        if matches!(who,Who::Mob(id)if id>=super::dragon::SPECIAL) {
            return false;
        }
        let data = arcadian_motion::data(p.id);
        let mut amount = t.damage;
        if let Some(owner) = p.owner_uuid {
            if self.arcadian_owner_ethereal(owner) {
                return true;
            }
        }
        let source = Damage {
            kind: DamageKind::Thrown,
            attacker: p.owner,
            from: Some([p.previous.x, p.previous.y, p.previous.z]),
        };
        let icy = data["icy"].as_bool().unwrap_or(false);
        let sharp = t.id == "arcadiandream:star_bullet";
        match who {
            Who::Mob(id) => {
                let fairy_owner = matches!(p.owner,Some(Who::Mob(owner))if self.mobs.iter().any(|m|m.id==owner&&m.kind.id.starts_with("arcadiandream:")));
                let mut landed = false;
                if let Some(m) = self.mobs.iter_mut().find(|m| m.id == id) {
                    if fairy_owner && m.kind.id.starts_with("arcadiandream:") {
                        return true;
                    }
                    if m.has_effect("arcadiandream:ethereal") {
                        return true;
                    }
                    if t.id == "arcadiandream:amulet_bullet" && m.kind.flags.undead {
                        amount *= 2.0;
                    }
                    let protection = m
                        .equipment_stacks()
                        .iter()
                        .flatten()
                        .map(|s| s.enchantment_level("arcadiandream:danmaku_protection") * 2)
                        .sum::<u32>()
                        .min(20);
                    amount *= 1.0 - protection as f32 / 25.0;
                    let mut armor = |_: f32| 0.0;
                    let hurt = if sharp {
                        m.damage_with(amount, source, Some(&mut armor))
                    } else {
                        m.damage(amount, source)
                    };
                    landed = hurt;
                    if icy && hurt {
                        m.ticks_frozen = (m.ticks_frozen + (10.0 * amount) as i32).min(560);
                    }
                    if hurt && p.owner == Some(Who::Player) {
                        self.last_hurt_by_player.insert(id, self.lan_acting);
                    }
                }
                if landed && p.owner == Some(Who::Player) {
                    if let Some(m) = self.mobs.iter().find(|m| m.id == id) {
                        let dead = m.health <= 0.0;
                        let victim = self.mob_entity(m);
                        let direct = horizon_client::advancements::EntityView {
                            kind: t.id.into(),
                            position: [p.position.x, p.position.y, p.position.z],
                            ..Default::default()
                        };
                        let kind = if sharp {
                            "arcadiandream:danmaku_sharp"
                        } else {
                            "arcadiandream:danmaku"
                        };
                        let damage = horizon_client::advancements::DamageView {
                            kind: kind.into(),
                            is_direct: false,
                            direct: Some(direct),
                            source: Some(self.player_entity()),
                            original: amount,
                            actual: amount,
                            blocked: false,
                        };
                        self.trigger_player_hurt_entity(&victim, &damage);
                        if dead {
                            self.trigger_killed(&victim, Some(&damage));
                        }
                    }
                }
            }
            Who::Player => {
                if self.combat.refuse_player_hit || self.spectator() {
                    return true;
                }
                self.combat.projectile_owner = if let Some(Who::Mob(id)) = p.owner {
                    Some(id)
                } else {
                    None
                };
                let kind = if sharp {
                    "arcadiandream:danmaku_sharp"
                } else {
                    "arcadiandream:danmaku"
                };
                // The host reads the mod's damage definitions, including sharp's armour bypass.
                let known = if let Some(Who::Mob(id)) = p.owner {
                    Some(Some(id))
                } else {
                    Some(None)
                };
                self.hurt_player_from(amount, source.from, kind, known);
                if icy {
                    if let Some(p) = self.player.as_mut() {
                        p.ticks_frozen = (p.ticks_frozen + (10.0 * amount) as i32).min(560);
                    }
                }
            }
        }
        true
    }
    pub(super) fn arcadian_special_melee(&mut self, stack: &ItemStack, entity: u32) {
        if let Some(m) = self.mobs.iter_mut().find(|m| m.id == entity) {
            if stack.item.as_ref() == "arcadiandream:nue_trident" {
                m.add_effect("minecraft:blindness", 100, 0);
            }
            if stack.item.as_ref() == "arcadiandream:mochi_mallet" && m.health <= 0.0 {
                if let Some(s) = self.held_mut() {
                    let mut data = custom(s);
                    let count = data["kill_count"].as_u64().unwrap_or(0) + 1;
                    data["kill_count"] = serde_json::json!(count);
                    super::arcadian::set_custom(s, data);
                    let n = count.min(500) as f64;
                    s.data_mut().set_loot_component("minecraft:attribute_modifiers",&serde_json::json!([
                        {"id":"arcadiandream:mochi_damage","type":"minecraft:attack_damage","amount":3.0+n/125.0,"operation":"add_value","slot":"mainhand"},
                        {"id":"arcadiandream:mochi_speed","type":"minecraft:attack_speed","amount":-3.2+n/416.666,"operation":"add_value","slot":"mainhand"}]));
                }
            }
        }
    }
    pub(super) fn arcadian_loot(
        &mut self,
        context: &horizon_client::mod_api::NativeLootDrops<'_>,
        drops: &mut Vec<ItemStack>,
    ) {
        if !self.native_on("arcadiandream") {
            return;
        }
        if let Some(tool) = context.tool.filter(|s| {
            s.item.starts_with("arcadiandream:hihiirokane_")
                && custom(s)["autosmelt"].as_bool().unwrap_or(true)
        }) {
            let _ = tool;
            for stack in drops.iter_mut() {
                if let Some(recipe) = self.recipes().cook("smelting", stack) {
                    let count = stack.count;
                    let mut out = recipe.result.clone();
                    out.count = out.count.saturating_mul(count);
                    *stack = out;
                }
            }
        }
        if context.table.ends_with("_leaves")
            && context.origin[1]
                >= number(
                    &super::arcadian::config()["hisouSwordOptions"],
                    "minHeightForPeaches",
                    196.0,
                )
            && context
                .tool
                .is_some_and(|s| s.item.as_ref() == "arcadiandream:hisou_sword")
        {
            use horizon_core::random::RandomSource;
            if self.world.random.next_float() < 0.1 {
                drops.push(ItemStack::new("arcadiandream:heavenly_peach", 1));
            }
        }
    }
}
