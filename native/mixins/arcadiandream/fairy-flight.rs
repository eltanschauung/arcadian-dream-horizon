fn hook(value:bool,m:&mut Mob,w:&MobWorld,events:&mut Vec<MobEvent>)->bool{
 if value&&m.no_ai&&m.kind.id.starts_with("arcadiandream:")&&m.kind.id.ends_with("fairy"){m.travel(w,horizon_core::geometry::Vec3::ZERO,events);}
 value
}
