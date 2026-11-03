fn hook(game: &mut crate::game::Game<'_>, pos: [i32; 3], entity: &mut ModBlockEntity) { game.arcadian_machine_tick(pos,entity); }
