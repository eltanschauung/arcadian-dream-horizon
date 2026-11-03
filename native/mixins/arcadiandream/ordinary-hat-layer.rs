fn hook(game: &crate::game::Game<'_>, mob: &NativeMobRender<'_>, who: Option<&NativePlayerRender<'_>>, draw: &mut BlockEntityDraw) { game.arcadian_draw_hat(mob,who.is_some_and(|p|p.local),draw); }
