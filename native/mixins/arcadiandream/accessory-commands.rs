fn hook(game: &mut crate::game::Game<'_>, commands: &mut Vec<NativeCommand>) { commands.push(NativeCommand::new("arcadian unequip <slot:word>")); }
