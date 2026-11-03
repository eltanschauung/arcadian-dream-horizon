#!/usr/bin/env python3
"""Build reproducible native descriptors and embedded original behavior data."""
from pathlib import Path
import json,re
ROOT=Path(__file__).resolve().parents[1];B=ROOT/'native/mixins/arcadiandream';A=ROOT/'assets/arcadiandream'
def main():
 # Geometry/animation data stays available to the compiled Rust renderer without external Java libraries.
 all_data={}
 for folder in ['geo','animations']:
  for p in sorted((A/folder).rglob('*.json')):all_data[str(p.relative_to(A))]=json.loads(p.read_text())
 recipes={p.stem:json.loads(p.read_text()) for p in sorted((ROOT/'data/arcadiandream/recipe/ritual_crafting').glob('*.json'))}
 behavior=json.loads((ROOT/'reference/behavior.json').read_text());shots=behavior['shots'];attacks=behavior['fairy_attacks']
 assert len(shots)==6 and sum(map(len,attacks.values()))==22
 for family,d in shots.items():
  item=ROOT/f'data/arcadiandream/horizon/item/{family}_shot.json';data=json.loads(item.read_text());data['components']['minecraft:max_damage']=d['durability'];data['components'].pop('minecraft:custom_data',None);item.write_text(json.dumps(data,indent=2)+'\n')
 data={'models':all_data,'rituals':recipes,'shots':shots,'fairy_attacks':attacks}
 (B/'assets.rs').write_text('// Derived from Maxmani and contributors: assets CC-BY-SA-4.0; data EUPL-1.2-or-later.\npub const DATA: &str = r####"'+json.dumps(data,separators=(',',':'))+'"####;\n')
 # References make entity textures available in the entity/GUI atlas to the native renderer.
 models=[]
 for p in sorted(list((A/'textures/entity').rglob('*.png'))+list((A/'textures/models').rglob('*.png'))):
  name=str(p.relative_to(A/'textures')).removesuffix('.png');model='arcadiandream:arcadian_render/'+name.replace('/','_');dest=A/'models'/('arcadian_render/'+name.replace('/','_')+'.json');dest.parent.mkdir(parents=True,exist_ok=True);dest.write_text(json.dumps({'textures':{'all':'arcadiandream:'+name},'elements':[{'from':[0,0,0],'to':[16,16,16],'faces':{'north':{'texture':'#all'}}}]}));models.append(model)
 (B/'standalone.rs').write_text('fn arcadian_models(models: &mut Vec<String>) { models.extend('+json.dumps(models).replace('[','[' ,1)+'.into_iter().map(str::to_owned)); }\n')
 # Namespaced mob registry rows use native-provided allay templates, then replace all AI/rendering.
 (B/'entity_types.rs').write_text('''fn entity_types(mods: &[ContentMod]) -> core::ops::ControlFlow<Vec<&EntityType>> {
    let mut result:Vec<&EntityType>=mods.iter().flat_map(|m|m.content.entity_types.iter()).collect();
    if result.iter().any(|t| t.id.starts_with("arcadiandream:")) {
        static TYPES: std::sync::OnceLock<Vec<EntityType>> = std::sync::OnceLock::new();
        let types = TYPES.get_or_init(|| ["fairy", "sunflower_fairy", "ice_fairy"].map(|name| EntityType {
            id: format!("arcadiandream:{name}"), template: "minecraft:allay".into(), item: format!("arcadiandream:{name}_spawn_egg"), damage: 0.0,
        }).into());
        result.extend(types); result.sort_by(|a,b| a.id.cmp(&b.id));
    }
    core::ops::ControlFlow::Break(result)
}
''')
 (B/'mob_types.rs').write_text('''fn mob_types(res:&crate::game::Resources) -> core::ops::ControlFlow<&HashMap<String, Arc<MobType>>> {
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
''')
 modules=[('equipment','crates/horizon-client/src/game.rs','arcadian_equipment','equipment.rs'),('settings','crates/horizon-world/src/loot.rs','arcadian_settings','config.rs'),('trades','crates/horizon-client/src/merchant.rs','arcadian_trades','trades.rs'),('trader-game','crates/horizon-client/src/game.rs','arcadian_trader','trader_game.rs'),('loot','crates/horizon-world/src/loot.rs','arcadian_loot','loot.rs'),('combat','crates/horizon-client/src/game.rs','arcadian_combat','combat.rs'),('config-menu','crates/horizon-client/src/game.rs','arcadian_config_menu','config_menu.rs'),('fairies','crates/horizon-client/src/game.rs','arcadian_fairies','fairies.rs'),('game','crates/horizon-client/src/game.rs','arcadian','runtime.rs'),('assets','crates/horizon-client/src/game.rs','arcadian_assets','assets.rs'),('render','crates/horizon-client/src/game.rs','arcadian_render','render.rs'),('machines','crates/horizon-client/src/game.rs','arcadian_machines','machines.rs'),('motion','crates/horizon-client/src/projectiles.rs','arcadian_motion','motion.rs')]
 manifest='[mod]\nid="arcadiandream"\nversion="0.1.0"\npriority=120\n[compatibility]\nhorizon_revision="d813f63a99978186b127ea14efe5d8603ac2ed4f"\n'
 for ident,file,name,source in modules:manifest+=f'\n[[module]]\nid="{ident}"\nfile="{file}"\nname="{name}"\nsource="{source}"\nvisibility="pub"\n'
 def hook(ident,target,helper,params,kind='HEAD',cancel=False):
  nonlocal manifest
  manifest+=f'\n[[injection]]\nid="{ident}"\nkind="{kind}"\ntarget="{target}"\nhelper="{helper}"\n'
  if cancel:manifest+='cancellable=true\n'
  manifest+='bindings=['+','.join('{name='+json.dumps(n)+',ty='+json.dumps(t)+',value='+json.dumps(v)+'}' for n,t,v in params)+']\n'
 def gamehook(name,params,call):
  source=name+'.rs';sig=', '.join(n+': '+t for n,t in params)
  (B/source).write_text(f'fn hook(game: &mut crate::game::Game<\'_>, {sig}) {{ {call} }}\n')
  hook(name,"horizon_client::game::native_hooks::Game<'_>::native_"+name,source,[('game',"&mut crate::game::Game<'_>",'&mut *self')]+[(n,t,'&mut *'+n if t.startswith('&mut') else n) for n,t in params])
 gamehook('use_item',[('stack','&ItemStack'),('claimed','&mut bool')], 'if game.arcadian_use(stack) { *claimed = true; }')
 gamehook('use_item_on',[('stack','&ItemStack'),('pos','[i32; 3]'),('face','[i32; 3]'),('claimed','&mut bool')], 'if game.arcadian_use_on(stack, pos, face) { *claimed = true; }')
 gamehook('player_tick',[], 'game.arcadian_player_tick();')
 gamehook('level_tick_pre',[('dimension','&str')], 'game.arcadian_world_tick(dimension);')
 gamehook('level_tick_post',[('dimension','&str')], 'game.arcadian_flush(dimension);')
 gamehook('hurt_enemy',[('stack','&ItemStack'),('entity','u32'),('kind','&str')], 'game.arcadian_hit(stack, entity, kind);')
 # Mutable Game is not passed to the pure descriptor/render hooks.
 hook('types','horizon_client_lib::mod_content::entity_types','entity_types.rs',[('mods','&[ContentMod]','mods')],cancel=True)
 hook('mob-types','horizon_client::game::Resources::mob_types','mob_types.rs',[('res','&crate::game::Resources','&*self')],cancel=True)
 hook('models',"horizon_client::game::native_hooks::Game<'_>::native_standalone_models",'standalone.rs',[('models','&mut Vec<String>','&mut *models')])
 (B/'listed.rs').write_text('fn listed(mods: &mut Vec<NativeMod>) { mods.push(NativeMod { id:"arcadiandream".into(), title:"Arcadian Dream 0.1.0".into(), description:"Unofficial native port of Maxmani and contributors’ Arcadian Dream. Code EUPL-1.2-or-later; assets CC-BY-SA-4.0.".into(), keys:Vec::new() }); }\n')
 hook('listed',"horizon_client::game::native_hooks::Game<'_>::native_mods",'listed.rs',[('mods','&mut Vec<NativeMod>','&mut *mods')])
 # Remaining hook sources are stable hand-written adapters.
 additional=json.loads((ROOT/'tools/hooks.json').read_text()) if (ROOT/'tools/hooks.json').exists() else []
 for h in additional:hook(**h)
 (B/'horizon-mixin.toml').write_text(manifest)
 print('Embedded',len(all_data),'model/animation definitions,',len(recipes),'rituals and',len(shots),'shot families')
if __name__=='__main__':main()
