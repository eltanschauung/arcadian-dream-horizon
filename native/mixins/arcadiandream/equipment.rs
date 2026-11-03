// SPDX-License-Identifier: EUPL-1.2-or-later
use super::{
    Game,
    arcadian::{config, custom, number, shot},
};
use horizon_client::{
    containers::BlockEntity,
    items::ItemStack,
    projectiles::{Projectile, ProjectileKind, arcadian_motion},
};
use horizon_core::{
    geometry::{Direction, Vec3},
    random::RandomSource,
};
impl Game<'_> {
    pub(super) fn arcadian_equipment_tick(&mut self) {
        if !self.native_on("arcadiandream") {
            return;
        }
        let Some(p) = self.native_player() else {
            return;
        };
        let at = [p.x.floor() as i32, p.y.floor() as i32, p.z.floor() as i32];
        let temp = self.biome_facts(at[0], at[1], at[2]).temperature;
        let nether = self.dim.id() == "minecraft:the_nether";
        let daylight =
            self.native_day_time().rem_euclid(24000) < 12000 && self.native_light(at).0 >= 14;
        let melts = nether || (temp > 0.5 && daylight && self.world.random.next_float() < 0.04);
        for slot in [self.selected, 40] {
            let damage =
                (self.world.random.next_int_bound(3).unwrap() as f32 * temp).max(0.0) as u32;
            let instant = nether
                && config()["icicleSwordOptions"]["meltsInstantly"]
                    .as_bool()
                    .unwrap_or(true);
            let creative = self.creative;
            let cell = self.stack_in_mut(slot);
            let Some(s) = cell.as_mut() else { continue };
            if s.item.as_ref() == "arcadiandream:icicle_sword" && melts {
                if instant {
                    s.count = s.count.saturating_sub(1);
                } else if !creative {
                    s.damage = s.damage.saturating_add(damage);
                    if s.damage >= 128 {
                        s.count = 0;
                    }
                }
                if s.count == 0 {
                    *cell = None;
                    continue;
                }
            }
            if s.item.as_ref() == "arcadiandream:mochi_mallet" {
                let kills = if config()["mochiMalletOptions"]["lowViolence"]
                    .as_bool()
                    .unwrap_or(false)
                {
                    0
                } else {
                    custom(s)["kill_count"].as_u64().unwrap_or(0)
                };
                let stage = [0, 10, 25, 50, 75, 100, 250, 500, 750, 900, 1000]
                    .iter()
                    .rposition(|n| kills >= *n)
                    .unwrap_or(0);
                s.data_mut().set_loot_component(
                    "minecraft:item_model",
                    &serde_json::json!(format!("arcadiandream:mochi_mallet_{stage}")),
                );
            }
        }
    }
    pub(super) fn arcadian_sealed(&self, pos: [i32; 3]) -> bool {
        self.native_on("arcadiandream")
            && (&self.world.block_entities)
                .into_iter()
                .any(|(&(x, y, z), e)| {
                    matches!(e,BlockEntity::Mod(m)if m.id=="arcadiandream:mysterious_seal")
                        && (x - pos[0]).abs() <= 16
                        && (y - pos[1]).abs() <= 16
                        && (z - pos[2]).abs() <= 16
                })
    }

    pub(super) fn arcadian_dispense(
        &mut self,
        pos: [i32; 3],
        direction: Direction,
        stack: &ItemStack,
    ) -> Option<(Option<ItemStack>, i32)> {
        if !self.native_on("arcadiandream")
            || !stack.item.starts_with("arcadiandream:")
            || !stack.item.ends_with("_shot")
        {
            return None;
        }
        let family = stack
            .item
            .trim_start_matches("arcadiandream:")
            .trim_end_matches("_shot");
        let data = shot(stack);
        let mut remainder = stack.clone();
        let durability = number(&data, "durability", 100.0) as u32;
        if stack.damage >= durability.saturating_sub(1) {
            return Some((Some(remainder), 1001));
        }
        let base =
            horizon_client::projectiles::thrown_type(&format!("arcadiandream:{family}_bullet"))?;
        let power = number(&data, "power", 2.0) * number(config(), "danmakuDamageMultiplier", 1.0);
        let step = direction.normal();
        let speed = number(&data, "speed", 1.0);
        let mut p = Projectile::new(
            ProjectileKind::Thrown(arcadian_motion::damage_type(base, power as f32)),
            Vec3::new(
                pos[0] as f64 + 0.5 + step[0] as f64 * 0.7,
                pos[1] as f64 + 0.5 + step[1] as f64 * 0.7,
                pos[2] as f64 + 0.5 + step[2] as f64 * 0.7,
            ),
            None,
        );
        p.no_gravity = true;
        p.velocity = Vec3::new(
            step[0] as f64 * speed,
            step[1] as f64 * speed,
            step[2] as f64 * speed,
        );
        self.add_projectile(p);
        if let Some(p) = self.creatures.projectiles.last() {
            arcadian_motion::register(p.id, data);
        }
        remainder.damage += 1;
        Some((Some(remainder), 1000))
    }
    pub(super) fn arcadian_tooltip(
        &self,
        stack: &ItemStack,
        lines: &mut Vec<(horizon_client::text::Text, u32)>,
    ) {
        if !stack.item.starts_with("arcadiandream:") {
            return;
        }
        let text = if stack.item.ends_with("_shot") {
            let d = shot(stack);
            format!(
                "Power {:.2}  Speed {:.2}  Duration {:.0}  Density {:.0}  Cooldown {:.0}",
                number(&d, "power", 2.0),
                number(&d, "speed", 0.8),
                number(&d, "duration", 100.0),
                number(&d, "density", 1.0),
                number(&d, "cooldown", 5.0)
            )
        } else if stack.item.as_ref() == "arcadiandream:mochi_mallet" {
            format!(
                "Kills: {}",
                custom(stack)["kill_count"].as_u64().unwrap_or(0)
            )
        } else if stack.item.starts_with("arcadiandream:hihiirokane_") {
            format!(
                "Autosmelt: {} (sneak + use with empty offhand)",
                custom(stack)["autosmelt"].as_bool().unwrap_or(true)
            )
        } else if [
            "arcadiandream:extend_item",
            "arcadiandream:magatama_necklace",
            "arcadiandream:fairy_charm",
        ]
        .contains(&stack.item.as_ref())
        {
            "Use to equip; /arcadian unequip life|necklace|charm to remove".into()
        } else {
            return;
        };
        lines.push((horizon_client::text::Text::literal(text), 0xffaaaaaa));
    }
}
