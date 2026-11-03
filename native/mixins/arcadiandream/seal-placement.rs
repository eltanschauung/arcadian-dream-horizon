fn hook(game: &mut crate::game::Game<'_>, stack: &ItemStack, pos: [i32;3], face: [i32;3], claimed: &mut bool) { if game.arcadian_place_seal(stack,pos,face){*claimed=true;} }
