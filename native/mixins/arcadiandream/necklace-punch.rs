fn hook(game:&mut crate::game::Game<'_>,i:usize)->core::ops::ControlFlow<()>{if game.arcadian_punch(i){core::ops::ControlFlow::Break(())}else{core::ops::ControlFlow::Continue(())}}
