fn hook(game: &mut crate::game::Game<'_>, mod_id: &str, buttons: &mut Vec<String>) { if mod_id=="arcadiandream"{buttons.extend(crate::game::arcadian_config_menu::buttons());} }
