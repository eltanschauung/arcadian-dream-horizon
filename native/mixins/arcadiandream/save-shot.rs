fn hook(p:&Projectile,uuid:[i32;4],owner:Option<[i32;4]>,raw:Option<&Tag>)->core::ops::ControlFlow<Option<Tag>> {
    thread_local!{static INSIDE:std::cell::Cell<bool>=const{std::cell::Cell::new(false)};}
    if !p.kind.id().starts_with("arcadiandream:")||!p.kind.id().ends_with("_bullet")||INSIDE.with(|v|v.get()){return core::ops::ControlFlow::Continue(())}
    INSIDE.with(|v|v.set(true));struct Guard;impl Drop for Guard{fn drop(&mut self){INSIDE.with(|v|v.set(false));}}let _guard=Guard;
    let mut result=crate::entity_nbt::throwable_to_nbt(p,uuid,owner,raw);
    if let Some(Tag::Compound(c))=&mut result {let v=crate::projectiles::arcadian_motion::data(p.id);c.insert("arcadiandream:shot".into(),Tag::String(v.to_string().as_str().into()));c.insert("arcadiandream:age".into(),Tag::Int(p.age as i32));}
    core::ops::ControlFlow::Break(result)
}
