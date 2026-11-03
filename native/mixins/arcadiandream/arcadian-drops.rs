fn hook(game: &mut crate::game::Game<'_>, context: &NativeLootDrops<'_>, drops: &mut Vec<ItemStack>) { game.arcadian_loot(context,drops); }
