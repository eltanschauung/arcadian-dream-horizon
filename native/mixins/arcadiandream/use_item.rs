fn hook(game: &mut crate::game::Game<'_>, stack: &ItemStack, claimed: &mut bool) { if game.arcadian_use(stack) { *claimed = true; } }
