fn hook(game:&mut crate::game::Game<'_>)->core::ops::ControlFlow<()>{if game.arcadian_totem_ready(){core::ops::ControlFlow::Break(())}else{core::ops::ControlFlow::Continue(())}}
