// SPDX-License-Identifier: EUPL-1.2-or-later
use super::Game;
use horizon_client::{merchant::arcadian_trades, mobs::Species};
impl Game<'_> {
    pub(super) fn arcadian_trader(&mut self, index: usize) -> bool {
        if !self.native_on("arcadiandream") || index >= self.mobs.len() {
            return false;
        }
        let m = &self.mobs[index];
        if m.species() != Species::Villager || m.extra.trade.as_ref().is_some_and(|t| !t.unset) {
            return false;
        }
        let job = m
            .extra
            .home
            .is_some_and(|p| self.native_block(p) == "arcadiandream:danmaku_crafting_table");
        let old = m
            .extra
            .trade
            .as_ref()
            .is_some_and(|t| t.xp > 0 && arcadian_trades::antiquarian(&t.offers));
        if !job && !old {
            return false;
        }
        let m = &mut self.mobs[index];
        let level = i32::from(m.extra.variant3.max(1));
        let mut offers = Vec::new();
        for l in 1..=level {
            offers.extend(arcadian_trades::level(l, &mut m.random));
        }
        let trade = m.extra.trade.get_or_insert_default();
        trade.offers = offers;
        trade.unset = false;
        if m.custom_name.is_none() {
            m.custom_name = Some("Antiquarian".into());
        }
        true
    }
    pub(super) fn arcadian_wandering(&mut self, index: usize) {
        if self.native_on("arcadiandream") && self.mobs[index].species() == Species::WanderingTrader
        {
            let m = &mut self.mobs[index];
            let offers = arcadian_trades::wandering(&mut m.random);
            m.extra.trade.get_or_insert_default().offers.extend(offers);
        }
    }
}
