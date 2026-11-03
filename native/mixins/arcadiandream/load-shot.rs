fn hook(tag:&Tag){
    if let (Some(Tag::String(s)),Some(kind),Some(pos))=(tag.get("arcadiandream:shot"),crate::entity_nbt::kind(tag).and_then(|id|crate::entity_nbt::throwable_kind(&id)),crate::entity_nbt::position(tag)) {
        if let Ok(v)=serde_json::from_str::<serde_json::Value>(&s.to_string_lossy()){let mut p=Projectile::new(kind,pos,None);p.velocity=crate::entity_nbt::motion(tag);let age=match tag.get("arcadiandream:age"){Some(Tag::Int(n))=>(*n).max(0)as u32,_=>0};crate::projectiles::arcadian_motion::restore(&mut p,v,age);}
    }
}
