from pathlib import Path
import json
R=Path(__file__).resolve().parents[1];B=R/'native/mixins/arcadiandream';hooks=[]
def add(ident,target,code,params,kind='HEAD',cancel=False):
 (B/(ident+'.rs')).write_text(code+'\n');hooks.append(dict(ident=ident,target=target,helper=ident+'.rs',params=params,kind=kind,cancel=cancel))
def game(ident,name,code,params,mutable=True):
 ty="&mut crate::game::Game<'_>" if mutable else "&crate::game::Game<'_>";gameparam=('game',ty,'&mut *self' if mutable else '&*self');sig=', '.join(n+': '+t for n,t,v in [gameparam]+params)
 add(ident,"horizon_client::game::native_hooks::Game<'_>::native_"+name,'fn hook('+sig+') { '+code+' }',[gameparam]+params)
game('machine-click','right_click_block','if game.arcadian_machine_click(pos) { *canceled=true; }',[('pos','[i32; 3]','pos'),('canceled','&mut bool','&mut *canceled')])
add('machine-types',"horizon_client::game::native_hooks::Game<'_>::native_block_entity",'fn hook(block:&str,entity:&mut Option<(String,Compound)>){crate::game::arcadian_machines::entity(block,entity);}',[('block','&str','block'),('entity','&mut Option<(String,Compound)>','&mut *entity')])
add('machine-tickers',"horizon_client::game::native_hooks::Game<'_>::native_block_entity_ticks",'fn hook(block:&str,ticks:&mut bool){if block.starts_with("arcadiandream:"){*ticks=true;}}',[('block','&str','block'),('ticks','&mut bool','&mut *ticks')])
add('machine-container',"horizon_client::game::native_hooks::Game<'_>::native_block_entity_container",'fn hook(block:&str,container:&mut Option<ModContainer>){crate::game::arcadian_machines::container(block,container);}',[('block','&str','block'),('container','&mut Option<ModContainer>','&mut *container')])
add('machine-screen',"horizon_client::game::native_hooks::Game<'_>::native_screen",'fn hook(block:&str,screen:&mut Option<NativeScreen>){crate::game::arcadian_machines::screen(block,screen);}',[('block','&str','block'),('screen','&mut Option<NativeScreen>','&mut *screen')])
game('machine-tick','block_entity_tick','game.arcadian_machine_tick(pos,entity);',[('pos','[i32; 3]','pos'),('entity','&mut ModBlockEntity','&mut *entity')])
game('fairy-render','mob_submit','if game.arcadian_draw_fairy(mob,draw) { *drawn=true; }',[('mob',"&NativeMobRender<'_>",'mob'),('draw','&mut BlockEntityDraw','&mut *draw'),('drawn','&mut bool','&mut *drawn')],False)
add('bullet-motion',"horizon_client_lib::projectiles::Projectile::tick_inner",'fn hook(p:&mut Projectile,w:&impl ProjectileWorld,impact:bool)->core::ops::ControlFlow<(bool,Hit)>{match crate::projectiles::arcadian_motion::tick(p,w,impact){Some(v)=>core::ops::ControlFlow::Break(v),None=>core::ops::ControlFlow::Continue(())}}',[('p','&mut Projectile','&mut *self'),('w','&impl ProjectileWorld','w'),('impact','bool','impact')],cancel=True)
add('hourai-damage',"horizon_client::game::creatures::Game<'_>::hurt_player_from",'fn hook(game:&mut crate::game::Game<\'_>,amount:f32,damage_type:&str)->core::ops::ControlFlow<bool>{match game.arcadian_hurt(amount,damage_type){Some(v)=>core::ops::ControlFlow::Break(v),None=>core::ops::ControlFlow::Continue(())}}',[('game',"&mut crate::game::Game<'_>",'&mut *self'),('amount','f32','amount'),('damage_type','&str','damage_type')],cancel=True)
game('item-finish','living_entity_use_item_finish','game.arcadian_finish(stack,result);',[('stack','&ItemStack','stack'),('result','&mut Option<ItemStack>','&mut *result')])
game('ethereal-block','left_click_block','if game.arcadian_ethereal(){*canceled=true;}',[('canceled','&mut bool','&mut *canceled')])
game('ethereal-attack','attack_entity','if game.arcadian_ethereal(){*canceled=true;}',[('canceled','&mut bool','&mut *canceled')])
game('entity-metadata','net_from_server','game.arcadian_receive(channel,data);',[('channel','&str','channel'),('data','&[u8]','data')])
add('bullet-item',"horizon_client_lib::projectiles::Projectile::drawn_item",'fn hook(p:&Projectile)->core::ops::ControlFlow<Option<crate::items::ItemStack>>{match crate::projectiles::arcadian_motion::drawn(p){Some(v)=>core::ops::ControlFlow::Break(Some(v)),None=>core::ops::ControlFlow::Continue(())}}',[('p','&Projectile','&*self')],cancel=True)
(R/'tools/hooks.json').write_text(json.dumps(hooks,indent=2)+'\n')
# Persistent projectile metadata is attached by the native save/load adapters.
add('save-shot', 'horizon_client_lib::entity_nbt::throwable_to_nbt', '''fn hook(p:&Projectile,uuid:[i32;4],owner:Option<[i32;4]>,raw:Option<&Tag>)->core::ops::ControlFlow<Option<Tag>> {
    thread_local!{static INSIDE:std::cell::Cell<bool>=const{std::cell::Cell::new(false)};}
    if !p.kind.id().starts_with("arcadiandream:")||!p.kind.id().ends_with("_bullet")||INSIDE.with(|v|v.get()){return core::ops::ControlFlow::Continue(())}
    INSIDE.with(|v|v.set(true));struct Guard;impl Drop for Guard{fn drop(&mut self){INSIDE.with(|v|v.set(false));}}let _guard=Guard;
    let mut result=crate::entity_nbt::throwable_to_nbt(p,uuid,owner,raw);
    if let Some(Tag::Compound(c))=&mut result {let v=crate::projectiles::arcadian_motion::data(p.id);c.insert("arcadiandream:shot".into(),Tag::String(v.to_string().as_str().into()));c.insert("arcadiandream:age".into(),Tag::Int(p.age as i32));}
    core::ops::ControlFlow::Break(result)
}''',[('p','&Projectile','p'),('uuid','[i32; 4]','uuid'),('owner','Option<[i32; 4]>','owner'),('raw','Option<&Tag>','raw')],cancel=True)
add('load-shot','horizon_client_lib::entity_nbt::throwable_from_nbt','''fn hook(tag:&Tag){
    if let (Some(Tag::String(s)),Some(kind),Some(pos))=(tag.get("arcadiandream:shot"),crate::entity_nbt::kind(tag).and_then(|id|crate::entity_nbt::throwable_kind(&id)),crate::entity_nbt::position(tag)) {
        if let Ok(v)=serde_json::from_str::<serde_json::Value>(&s.to_string_lossy()){let mut p=Projectile::new(kind,pos,None);p.velocity=crate::entity_nbt::motion(tag);let age=match tag.get("arcadiandream:age"){Some(Tag::Int(n))=>(*n).max(0)as u32,_=>0};crate::projectiles::arcadian_motion::restore(&mut p,v,age);}
    }
}''',[('tag','&Tag','tag')])
(R/'tools/hooks.json').write_text(json.dumps(hooks,indent=2)+'\n')

game('arcadian-settings','settings','if mod_id=="arcadiandream"{buttons.extend(crate::game::arcadian_config_menu::buttons());}',[('mod_id','&str','mod_id'),('buttons','&mut Vec<String>','&mut *buttons')])
game('arcadian-settings-click','settings_click','if mod_id=="arcadiandream"{crate::game::arcadian_config_menu::click(button);}',[('mod_id','&str','mod_id'),('button','usize','button')])
(R/'tools/hooks.json').write_text(json.dumps(hooks,indent=2)+'\n')

add('death-protection-ready',"horizon_client::game::totem::Game<'_>::update_totem_ready",'fn hook(game:&mut crate::game::Game<\'_>)->core::ops::ControlFlow<()>{if game.arcadian_totem_ready(){core::ops::ControlFlow::Break(())}else{core::ops::ControlFlow::Continue(())}}',[('game',"&mut crate::game::Game<'_>",'&mut *self')],cancel=True)
add('death-protection-apply',"horizon_client::game::totem::Game<'_>::apply_totem",'fn hook(game:&mut crate::game::Game<\'_>)->core::ops::ControlFlow<Option<Body>>{if let Some((at,width,height))=game.arcadian_resurrect(){core::ops::ControlFlow::Break(Some(Body{at,width,height}))}else{core::ops::ControlFlow::Continue(())}}',[('game',"&mut crate::game::Game<'_>",'&mut *self')],cancel=True)
game('accessory-commands','register_commands','commands.push(NativeCommand::new("arcadian unequip <slot:word>"));',[('commands','&mut Vec<NativeCommand>','&mut *commands')])
game('accessory-command','perform_command','if game.arcadian_command(player.uuid,command){*ran=true;}',[('player','&NativeServerPlayer','player'),('command','&str','command'),('ran','&mut bool','&mut *ran')])
add('scythe-duration',"horizon_client::game::native_hooks::Game<'_>::native_use_duration",'fn hook(stack:&ItemStack,duration:&mut i32){if stack.item.as_ref()=="arcadiandream:death_scythe"{*duration=72000;}}',[('stack','&ItemStack','stack'),('duration','&mut i32','&mut *duration')])
add('scythe-animation',"horizon_client::game::native_hooks::Game<'_>::native_use_animation",'fn hook(stack:&ItemStack,animation:&mut UseAnimation){if stack.item.as_ref()=="arcadiandream:death_scythe"{*animation=UseAnimation::Bow;}}',[('stack','&ItemStack','stack'),('animation','&mut UseAnimation','&mut *animation')])
game('scythe-release','release_using','if game.arcadian_release(stack,time_left){*used=true;}',[('stack','&ItemStack','stack'),('time_left','i32','time_left'),('used','&mut bool','&mut *used')])
(R/'tools/hooks.json').write_text(json.dumps(hooks,indent=2)+'\n')

add('danmaku-impact',"horizon_client::game::creatures::Game<'_>::projectile_hit",'fn hook(game:&mut crate::game::Game<\'_>,p:&Projectile,hit:Hit)->core::ops::ControlFlow<Result<()>>{if game.arcadian_projectile_hit(p,hit){core::ops::ControlFlow::Break(Ok(()))}else{core::ops::ControlFlow::Continue(())}}',[('game',"&mut crate::game::Game<'_>",'&mut *self'),('p','&Projectile','p'),('hit','Hit','hit')],cancel=True)
game('arcadian-drops','loot_drops','game.arcadian_loot(context,drops);',[('context',"&NativeLootDrops<'_>",'context'),('drops','&mut Vec<ItemStack>','&mut *drops')])
(R/'tools/hooks.json').write_text(json.dumps(hooks,indent=2)+'\n')
add('table-slot-rules','horizon_client_lib::menu::native_menu_slot_rules','''fn hook(menu:&str,slot:usize,stack:&ItemStack,may_place:&mut bool,max_stack:&mut u32){
    if menu!="arcadiandream:danmaku_crafting_table"||slot>=7{return}
    let item=stack.item.as_ref();*may_place=match slot{
        0=>item.starts_with("arcadiandream:")&&item.ends_with("_bullet_core"),
        1=>item.starts_with("arcadiandream:")&&item.ends_with("_shot"),2=>false,
        3=>["arcadiandream:power_item","arcadiandream:big_power_item","arcadiandream:point_item","arcadiandream:star_item","arcadiandream:time_orb","arcadiandream:enchanted_ice"].contains(&item),
        4=>item=="arcadiandream:faith_item",5=>item.starts_with("arcadiandream:")&&item.ends_with("_pattern"),
        6=>item.starts_with("minecraft:")&&item.ends_with("_dye"),_=>false};if slot==1{*max_stack=1;}
}''',[('menu','&str','menu'),('slot','usize','slot'),('stack','&ItemStack','stack'),('may_place','&mut bool','&mut *may_place'),('max_stack','&mut u32','&mut *max_stack')])
(R/'tools/hooks.json').write_text(json.dumps(hooks,indent=2)+'\n')

add('arcadian-loot-root','horizon_world::loot::shared_tables','fn hook(root:&std::path::Path){crate::loot::arcadian_loot::root(root);}',[('root','&std::path::Path','root')])
add('arcadian-loot-tables','horizon_world::loot::native_loot_table_load','fn hook(id:&str,table:&mut serde_json::Value){crate::loot::arcadian_loot::extend(id,table);}',[('id','&str','id'),('table','&mut serde_json::Value','&mut *table')])
(R/'tools/hooks.json').write_text(json.dumps(hooks,indent=2)+'\n')

game('arcadian-save-stop','server_stopping','crate::game::arcadian::save_now();',[])
game('arcadian-save-logout','player_logged_out','crate::game::arcadian::save_now();',[])
add('bullet-pickable',"horizon_client::game::deflect::attackable",'fn hook(p:&Projectile)->core::ops::ControlFlow<bool>{if p.kind.id().starts_with("arcadiandream:")&&p.kind.id().ends_with("_bullet"){core::ops::ControlFlow::Break(true)}else{core::ops::ControlFlow::Continue(())}}',[('p','&Projectile','p')],cancel=True)
add('necklace-pick',"horizon_client::game::deflect::Game<'_>::redirectable_hit",'''fn hook(game:&crate::game::Game<'_>)->core::ops::ControlFlow<Option<(usize,f64)>>{
 if !game.native_on("arcadiandream"){return core::ops::ControlFlow::Continue(())}
 let result=(||{let (eye,dir)=game.aim()?;game.creatures.projectiles.iter().enumerate().filter(|(_,p)|attackable(p)&&(!p.kind.id().starts_with("arcadiandream:")||(game.arcadian_can_punch()&&!game.arcadian_own_shot(p)))).filter_map(|(i,p)|ray_box(eye,dir,game.entity_reach,pick_box(p)).map(|t|(i,t))).min_by(|a,b|a.1.total_cmp(&b.1))})();core::ops::ControlFlow::Break(result)
}''',[('game',"&crate::game::Game<'_>",'&*self')],cancel=True)
add('necklace-punch',"horizon_client::game::deflect::Game<'_>::deflect",'fn hook(game:&mut crate::game::Game<\'_>,i:usize)->core::ops::ControlFlow<()>{if game.arcadian_punch(i){core::ops::ControlFlow::Break(())}else{core::ops::ControlFlow::Continue(())}}',[('game',"&mut crate::game::Game<'_>",'&mut *self'),('i','usize','i')],cancel=True)
(R/'tools/hooks.json').write_text(json.dumps(hooks,indent=2)+'\n')

add('antiquarian-workstation','horizon_client_lib::mobs::villager::job_site_profession','fn hook(block:&str)->core::ops::ControlFlow<Option<u8>>{if block=="arcadiandream:danmaku_crafting_table"{core::ops::ControlFlow::Break(Some(9))}else{core::ops::ControlFlow::Continue(())}}',[('block','&str','block')],cancel=True)
add('antiquarian-offers',"horizon_client::game::trading::Game<'_>::ensure_offers",'''fn hook(game:&mut crate::game::Game<'_>,index:usize)->core::ops::ControlFlow<()>{
 thread_local!{static INSIDE:std::cell::Cell<bool>=const{std::cell::Cell::new(false)};}
 if INSIDE.with(|v|v.get()){return core::ops::ControlFlow::Continue(())}
 if game.arcadian_trader(index){return core::ops::ControlFlow::Break(())}
 if !game.native_on("arcadiandream")||!game.mobs[index].extra.trade.as_ref().is_none_or(|t|t.unset){return core::ops::ControlFlow::Continue(())}
 INSIDE.with(|v|v.set(true));struct Guard;impl Drop for Guard{fn drop(&mut self){INSIDE.with(|v|v.set(false));}}let _guard=Guard;
 game.ensure_offers(index);game.arcadian_wandering(index);core::ops::ControlFlow::Break(())
}''',[('game',"&mut crate::game::Game<'_>",'&mut *self'),('index','usize','index')],cancel=True)
add('antiquarian-level','horizon_client_lib::merchant::Session::notify_trade','fn hook(session:&mut Session,index:usize){crate::merchant::arcadian_trades::level_up(session,index);}',[('session','&mut Session','&mut *self'),('index','usize','index')])
(R/'tools/hooks.json').write_text(json.dumps(hooks,indent=2)+'\n')

add('hourai-effect-immunity',"horizon_client::game::potions::Game<'_>::add_player_effect",'fn hook(game:&mut crate::game::Game<\'_>,effect:&EffectInstance)->core::ops::ControlFlow<bool>{if game.arcadian_effect_immune(effect.id){core::ops::ControlFlow::Break(false)}else{core::ops::ControlFlow::Continue(())}}',[('game',"&mut crate::game::Game<'_>",'&mut *self'),('effect','&EffectInstance','&effect')],cancel=True)
(R/'tools/hooks.json').write_text(json.dumps(hooks,indent=2)+'\n')

add('nue-impaling','horizon_items::enchant::Enchantments::can_enchant','''fn hook(id:&str,item:&str)->core::ops::ControlFlow<bool>{if (item.starts_with("arcadiandream:")&&item.ends_with("_shot"))||item=="arcadiandream:hourai_elixir"{return core::ops::ControlFlow::Break(false)} if id=="minecraft:impaling"&&item=="arcadiandream:nue_trident"{core::ops::ControlFlow::Break(horizon_world::loot::arcadian_settings::config()["nueTridentOptions"]["canHaveImpaling"].as_bool().unwrap_or(true))}else{core::ops::ControlFlow::Continue(())}}''',[('id','&str','id'),('item','&str','item')],cancel=True)
(R/'tools/hooks.json').write_text(json.dumps(hooks,indent=2)+'\n')

game('arcadian-dispensers','dispense_item','if let Some(v)=game.arcadian_dispense(pos,facing,stack){*answer=Some(v);}',[('pos','[i32;3]','pos'),('facing','Direction','facing'),('stack','&ItemStack','stack'),('answer','&mut Option<NativeDispensed>','&mut *answer')])
game('seal-placement','use_item_on','if game.arcadian_place_seal(stack,pos,face){*claimed=true;}',[('stack','&ItemStack','stack'),('pos','[i32;3]','pos'),('face','[i32;3]','face'),('claimed','&mut bool','&mut *claimed')])
game('arcadian-tooltips','tooltip','game.arcadian_tooltip(stack,lines);',[('stack','&ItemStack','stack'),('lines','&mut Vec<(Text,u32)>','&mut *lines')],False)
add('seal-tnt',"horizon_client::game::tnt::Game<'_>::spawn_primed_tnt",'''fn hook(game:&mut crate::game::Game<'_>,pos:BlockPos,short:bool)->core::ops::ControlFlow<()>{if !short&&game.arcadian_sealed([pos.x,pos.y,pos.z]){game.arcadian_queue(format!("execute in {} run setblock {} {} {} minecraft:tnt",game.dim.id(),pos.x,pos.y,pos.z));core::ops::ControlFlow::Break(())}else{core::ops::ControlFlow::Continue(())}}''',[('game',"&mut crate::game::Game<'_>",'&mut *self'),('pos','BlockPos','pos'),('short','bool','short')],cancel=True)
add('hisou-shield','horizon_world::item_defaults::native_modify_default_components','fn hook(item:&str,components:&mut serde_json::Map<String,serde_json::Value>){if item=="arcadiandream:hisou_sword"{let seconds=if crate::loot::arcadian_settings::config()["hisouSwordOptions"]["canDisableShields"].as_bool().unwrap_or(true){5.0}else{0.0};components.insert("minecraft:weapon".into(),serde_json::json!({"item_damage_per_attack":1,"disable_blocking_for_seconds":seconds}));}}',[('item','&str','item'),('components','&mut serde_json::Map<String,serde_json::Value>','&mut *components')])
(R/'tools/hooks.json').write_text(json.dumps(hooks,indent=2)+'\n')

add('arcadian-player-layer-listener',"horizon_client::game::native_hooks::Game<'_>::native_player_render_listened",'fn hook(listened:&mut bool){*listened=true;}',[('listened','&mut bool','&mut *listened')])
game('ordinary-hat-layer','mob_layer','game.arcadian_draw_hat(mob,who.is_some_and(|p|p.local),draw);',[('mob',"&NativeMobRender<'_>",'mob'),('who',"Option<&NativePlayerRender<'_>>",'who'),('draw','&mut BlockEntityDraw','&mut *draw')],False)
(R/'tools/hooks.json').write_text(json.dumps(hooks,indent=2)+'\n')

game('arcadian-reconnect','player_logged_in','crate::game::arcadian::reconnect(player);',[('player','u128','player')])
(R/'tools/hooks.json').write_text(json.dumps(hooks,indent=2)+'\n')

add('pedestal-update',"horizon_client::game::native_hooks::Game<'_>::native_block_entity_update_tag",'fn hook(entity:&ModBlockEntity,tag:&mut Option<Compound>){crate::game::arcadian_machines::update_tag(entity,tag);}',[('entity','&ModBlockEntity','entity'),('tag','&mut Option<Compound>','&mut *tag')])
game('pedestal-draw','block_entity_submit','game.arcadian_pedestal_draw(entity,draw);',[('entity','&ModBlockEntity','entity'),('draw','&mut BlockEntityDraw','&mut *draw')],False)
(R/'tools/hooks.json').write_text(json.dumps(hooks,indent=2)+'\n')

add('mob-type-registration','horizon_client::game::mods::register_entity_types','''fn hook(mods:&[horizon_client::mod_content::ContentMod])->core::ops::ControlFlow<()>{let types=horizon_client::mod_content::entity_types(mods);let _=horizon_client::projectiles::register_thrown_types(types.iter().filter(|t|t.template!="minecraft:allay").map(|t|(t.id.clone(),t.item.clone(),t.damage)));core::ops::ControlFlow::Break(())}''',[('mods','&[horizon_client::mod_content::ContentMod]','mods')],cancel=True)
(R/'tools/hooks.json').write_text(json.dumps(hooks,indent=2)+'\n')
add('seal-survival',"horizon_client::game::native_hooks::GameLevel<'_>::native_can_survive",'''fn hook(level:&crate::game::GameLevel<'_>,pos:BlockPos,state:BlockStateId,survives:&mut Option<bool>){
 let Some(s)=level.registry.state(state).filter(|s|s.name=="arcadiandream:mysterious_seal")else{return};
 let d=match s.properties.get("facing").map(String::as_str){Some("up")=>[0,-1,0],Some("down")=>[0,1,0],Some("east")=>[-1,0,0],Some("west")=>[1,0,0],Some("south")=>[0,0,-1],_=>[0,0,1]};
 *survives=Some(level.facts.get(level.world.block(pos.x+d[0],pos.y+d[1],pos.z+d[2])).full_collision);
}''',[('level',"&crate::game::GameLevel<'_>",'&*self'),('pos','BlockPos','pos'),('state','BlockStateId','state'),('survives','&mut Option<bool>','&mut *survives')])
add('seal-place-state',"horizon_client::game::native_hooks::GameLevel<'_>::native_state_for_placement",'''fn hook(level:&crate::game::GameLevel<'_>,state:BlockStateId,context:&PlacementContext,fluid:FluidState,answer:&mut Option<Option<BlockStateId>>){
 if level.native_state_block(state)!=Some("arcadiandream:mysterious_seal"){return}
 let facing=match context.face {Direction::Up=>"up",Direction::Down=>"down",Direction::East=>"east",Direction::West=>"west",Direction::South=>"south",Direction::North=>"north"};
 *answer=Some(level.native_state_with_property(state,"facing",facing).and_then(|s|level.native_state_with_property(s,"waterlogged",if fluid.kind()==FluidKind::Water{"true"}else{"false"})));
}''',[('level',"&crate::game::GameLevel<'_>",'&*self'),('state','BlockStateId','state'),('context','&PlacementContext','context'),('fluid','FluidState','fluid'),('answer','&mut Option<Option<BlockStateId>>','&mut *answer')])
hooks=[h for h in hooks if h['ident']!='seal-placement']
(R/'tools/hooks.json').write_text(json.dumps(hooks,indent=2)+'\n')
add('fairy-flight','horizon_client_lib::mobs::Mob::tick','''fn hook(value:bool,m:&mut Mob,w:&MobWorld,events:&mut Vec<MobEvent>)->bool{
 if value&&m.no_ai&&m.kind.id.starts_with("arcadiandream:")&&m.kind.id.ends_with("fairy"){m.travel(w,horizon_core::geometry::Vec3::ZERO,events);}
 value
}''',[('value','bool','$value'),('m','&mut Mob','&mut *self'),('w','&MobWorld','w'),('events','&mut Vec<MobEvent>','&mut *events')],kind='RETURN')
(R/'tools/hooks.json').write_text(json.dumps(hooks,indent=2)+'\n')
add('danmaku-damage-catalog','horizon_world::damage_source::Catalog::from_json',(B/'damage-catalog.rs').read_text().rstrip(),[('v','&serde_json::Value','v'),('resources','&serde_json::Value','resources')],cancel=True)
(R/'tools/hooks.json').write_text(json.dumps(hooks,indent=2)+'\n')
