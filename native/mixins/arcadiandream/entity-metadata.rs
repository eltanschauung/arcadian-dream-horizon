fn hook(game: &mut crate::game::Game<'_>, channel: &str, data: &[u8]) { game.arcadian_receive(channel,data); }
