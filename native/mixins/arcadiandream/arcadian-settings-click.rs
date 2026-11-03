fn hook(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) { if mod_id=="arcadiandream"{crate::game::arcadian_config_menu::click(button);} }
