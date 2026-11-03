fn hook(game: &mut crate::game::Game<'_>, pos: [i32; 3], canceled: &mut bool) { if game.arcadian_machine_click(pos) { *canceled=true; } }
