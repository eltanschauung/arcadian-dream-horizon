fn hook(game: &mut crate::game::Game<'_>, player: &NativeServerPlayer, command: &str, ran: &mut bool) { if game.arcadian_command(player.uuid,command){*ran=true;} }
