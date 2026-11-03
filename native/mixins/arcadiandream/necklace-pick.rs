fn hook(game:&crate::game::Game<'_>)->core::ops::ControlFlow<Option<(usize,f64)>>{
 if !game.native_on("arcadiandream"){return core::ops::ControlFlow::Continue(())}
 let result=(||{let (eye,dir)=game.aim()?;game.creatures.projectiles.iter().enumerate().filter(|(_,p)|attackable(p)&&(!p.kind.id().starts_with("arcadiandream:")||(game.arcadian_can_punch()&&!game.arcadian_own_shot(p)))).filter_map(|(i,p)|ray_box(eye,dir,game.entity_reach,pick_box(p)).map(|t|(i,t))).min_by(|a,b|a.1.total_cmp(&b.1))})();core::ops::ControlFlow::Break(result)
}
