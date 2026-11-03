fn hook(game: &crate::game::Game<'_>, mob: &NativeMobRender<'_>, draw: &mut BlockEntityDraw, drawn: &mut bool) { if game.arcadian_draw_fairy(mob,draw) { *drawn=true; } }
