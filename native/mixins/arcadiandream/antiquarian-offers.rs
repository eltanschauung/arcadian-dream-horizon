fn hook(game:&mut crate::game::Game<'_>,index:usize)->core::ops::ControlFlow<()>{
 thread_local!{static INSIDE:std::cell::Cell<bool>=const{std::cell::Cell::new(false)};}
 if INSIDE.with(|v|v.get()){return core::ops::ControlFlow::Continue(())}
 if game.arcadian_trader(index){return core::ops::ControlFlow::Break(())}
 if !game.native_on("arcadiandream")||!game.mobs[index].extra.trade.as_ref().is_none_or(|t|t.unset){return core::ops::ControlFlow::Continue(())}
 INSIDE.with(|v|v.set(true));struct Guard;impl Drop for Guard{fn drop(&mut self){INSIDE.with(|v|v.set(false));}}let _guard=Guard;
 game.ensure_offers(index);game.arcadian_wandering(index);core::ops::ControlFlow::Break(())
}
