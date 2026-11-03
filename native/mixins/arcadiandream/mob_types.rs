fn mob_types(res:&crate::game::Resources) -> core::ops::ControlFlow<&HashMap<String, Arc<MobType>>> {
    let result=&res.rest().mob_types;
    if horizon_client::projectiles::thrown_type("arcadiandream:circle_bullet").is_none(){return core::ops::ControlFlow::Continue(())}
    type Table=HashMap<String,Arc<MobType>>;
    static CACHE:std::sync::OnceLock<std::sync::Mutex<HashMap<usize,&'static Table>>>=std::sync::OnceLock::new();
    let Some(base)=result.get("minecraft:allay")else{return core::ops::ControlFlow::Continue(())};
    let key=Arc::as_ptr(base)as usize;
    let mut cache=CACHE.get_or_init(Default::default).lock().unwrap();
    let table=*cache.entry(key).or_insert_with(||{
        let mut table=result.clone();
        for (name,width,height,health) in [("fairy",0.55,1.55,10.0),("sunflower_fairy",0.7,1.7,20.0),("ice_fairy",0.55,1.55,16.0)]{
            let mut kind=(**base).clone();kind.id=format!("arcadiandream:{name}");kind.width=width;kind.height=height;kind.eye_height=height*0.8;kind.category="monster".into();kind.max_health=health;kind.speed=0.25;kind.flying_speed=0.5;kind.gravity=0.0;kind.sounds=["entity.bat.ambient","entity.bat.hurt","entity.bat.death"];kind.step_sound=None;let mut attrs=(*kind.attributes).clone();attrs.insert("minecraft:max_health".into(),health as f64);attrs.insert("minecraft:movement_speed".into(),0.25);attrs.insert("minecraft:flying_speed".into(),0.5);attrs.insert("minecraft:gravity".into(),0.0);kind.attributes=Arc::new(attrs);
            table.insert(kind.id.clone(),Arc::new(kind));
        }
        Box::leak(Box::new(table))
    });
    core::ops::ControlFlow::Break(table)
}
