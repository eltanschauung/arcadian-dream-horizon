fn hook(game: &mut crate::game::Game<'_>, stack: &ItemStack, time_left: i32, used: &mut bool) { if game.arcadian_release(stack,time_left){*used=true;} }
