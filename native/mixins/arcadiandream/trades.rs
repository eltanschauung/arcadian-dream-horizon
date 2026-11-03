// SPDX-License-Identifier: EUPL-1.2-or-later
// Arcadian Dream's original trade prices, experience and shot ranges.
use super::{ItemCost, MerchantOffer, Session};
use crate::items::ItemStack;
use horizon_core::random::RandomSource;
use serde_json::json;
fn offer(
    input: &str,
    count: u32,
    result: ItemStack,
    max: i32,
    xp: i32,
    price: f32,
) -> MerchantOffer {
    MerchantOffer {
        base_cost_a: ItemCost {
            stack: ItemStack::new(input, count),
        },
        cost_b: None,
        result,
        uses: 0,
        max_uses: max,
        reward_exp: true,
        special_price_diff: 0,
        demand: 0,
        price_multiplier: price,
        xp,
    }
}
fn shot(r: &mut impl RandomSource) -> ItemStack {
    let family = ["circle", "bubble", "amulet", "star", "kunai", "pellet"]
        [r.next_int_bound(6).unwrap() as usize];
    let mut s = ItemStack::new(&format!("arcadiandream:{family}_shot"), 1);
    let color = [
        16711680, 16753920, 16770048, 12582656, 65280, 65535, 3847130, 1908223, 10494192, 16711935,
        15961002,
    ][r.next_int_bound(11).unwrap() as usize];
    let data = json!({"power":10-r.next_int_bound(5).unwrap(),"speed":0.6+r.next_float()*0.6,"duration":40+r.next_int_bound(41).unwrap(),"cooldown":30+r.next_int_bound(31).unwrap(),"density":3+r.next_int_bound(6).unwrap(),"color":color});
    s.data_mut()
        .set_loot_component("minecraft:custom_data", &json!({"arcadiandream":data}));
    s.data_mut().dyed_color = Some(color);
    s
}
pub fn level(level: i32, r: &mut impl RandomSource) -> Vec<MerchantOffer> {
    let buy = |item: &str, n: u32, max: i32| {
        offer(
            &format!("arcadiandream:{item}"),
            n,
            ItemStack::new("minecraft:emerald", 1),
            max,
            1,
            0.05,
        )
    };
    let sell = |n: u32, item: &str, max: i32, xp: i32, price: f32| {
        offer(
            "minecraft:emerald",
            n,
            ItemStack::new(&format!("arcadiandream:{item}"), 1),
            max,
            xp,
            price,
        )
    };
    match level {
        1 => {
            let mut pool = vec![
                buy("star_item", 16, 12),
                buy("power_item", 8, 16),
                buy("point_item", 8, 16),
            ];
            pool.remove(r.next_int_bound(3).unwrap() as usize);
            pool
        }
        2 => vec![sell(12, "faith_item", 12, 10, 0.05)],
        3 => vec![sell(16, "healing_charm", 12, 20, 0.2)],
        4 => vec![offer("minecraft:emerald", 32, shot(r), 3, 30, 0.2)],
        5 => vec![
            sell(42, "bomb_item", 3, 30, 0.2),
            offer("minecraft:emerald", 32, shot(r), 3, 30, 0.2),
        ],
        _ => vec![],
    }
}
pub fn antiquarian(offers: &[MerchantOffer]) -> bool {
    offers
        .iter()
        .any(|o| o.base_cost_a.stack.item.starts_with("arcadiandream:"))
}
pub fn level_up(s: &mut Session, index: usize) {
    if !s.villager || !antiquarian(&s.offers) {
        return;
    }
    s.profession = "arcadiandream:antiquarian".into();
    if let Some(o) = s.offers.get(index) {
        if super::can_level_up(s.level) && s.xp + o.xp >= super::max_xp(s.level) {
            s.offers.extend(level(s.level + 1, &mut s.random));
        }
    }
}
pub fn wandering(r: &mut impl RandomSource) -> Vec<MerchantOffer> {
    let mut v = Vec::new();
    // The factories enter the trader's ordinary / special random pools.
    if r.next_int_bound(10).unwrap() == 0 {
        v.push(offer(
            "minecraft:emerald",
            12,
            ItemStack::new("arcadiandream:ghastly_lantern", 1),
            3,
            1,
            0.05,
        ));
    }
    if r.next_int_bound(6).unwrap() == 0 {
        v.push(offer(
            "minecraft:emerald",
            8,
            ItemStack::new("arcadiandream:life_fragment", 1),
            3,
            1,
            0.05,
        ));
    }
    v
}
