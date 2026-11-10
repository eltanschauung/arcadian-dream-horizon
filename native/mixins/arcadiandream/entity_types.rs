fn entity_types(mods: &[ContentMod]) -> core::ops::ControlFlow<Vec<&EntityType>> {
    let mut result:Vec<&EntityType>=mods.iter().flat_map(|m|m.content.entity_types.iter()).collect();
    if result.iter().any(|t| t.id.starts_with("arcadiandream:")) {
        static TYPES: std::sync::OnceLock<Vec<EntityType>> = std::sync::OnceLock::new();
        let types = TYPES.get_or_init(|| ["fairy", "sunflower_fairy", "ice_fairy"].map(|name| EntityType {
            id: format!("arcadiandream:{name}"), template: "minecraft:allay".into(), item: None, damage: 0.0, attributes: Default::default(),
        }).into());
        result.extend(types); result.sort_by(|a,b| a.id.cmp(&b.id));
    }
    core::ops::ControlFlow::Break(result)
}
