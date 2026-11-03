#!/usr/bin/env python3
"""Translate 1.20.1 resource codecs to Horizon's 26.3 data formats."""
from pathlib import Path
import json,shutil
ROOT=Path(__file__).resolve().parents[1]
def write(p,v):p.parent.mkdir(parents=True,exist_ok=True);p.write_text(json.dumps(v,indent=2,ensure_ascii=False)+'\n')
def ingredient(v):
 if isinstance(v,dict):
  if 'item'in v:return v['item']
  if 'tag'in v:return '#'+v['tag']
 if isinstance(v,list):return [ingredient(x)for x in v]
 return v
def main():
 for namespace in (ROOT/'data').iterdir():
  if not namespace.is_dir():continue
  for old,new in [('recipes','recipe'),('loot_tables','loot_table'),('advancements','advancement'),('structures','structure')]:
   p=namespace/old
   if p.exists():shutil.copytree(p,namespace/new,dirs_exist_ok=True);shutil.rmtree(p)
  tags=namespace/'tags'
  for old,new in [('items','item'),('blocks','block'),('entity_types','entity_type')]:
   p=tags/old
   if p.exists():shutil.copytree(p,tags/new,dirs_exist_ok=True);shutil.rmtree(p)
 for p in (ROOT/'data/arcadiandream/recipe').rglob('*.json'):
  d=json.loads(p.read_text())
  for k in ['ingredient','base','addition','template']:
   if k in d:d[k]=ingredient(d[k])
  if 'ingredients'in d:d['ingredients']=[ingredient(v)for v in d['ingredients']]
  if 'key'in d:d['key']={k:ingredient(v)for k,v in d['key'].items()}
  if isinstance(d.get('result'),dict)and'item'in d['result']:d['result']['id']=d['result'].pop('item')
  write(p,d)
 # Loot predicates/functions changed after 1.20.1. Keep Fortune/Silk Touch and Looting distinct.
 def loot(v):
  if isinstance(v,list):return [loot(x)for x in v]
  if not isinstance(v,dict):return v
  v={k:loot(x)for k,x in v.items()}
  if v.get('function')=='minecraft:looting_enchant':v['function']='minecraft:enchanted_count_increase';v['enchantment']='minecraft:looting'
  if v.get('condition')=='minecraft:random_chance_with_looting':
   chance=v.pop('chance',0);mult=v.pop('looting_multiplier',0);v.update(condition='minecraft:random_chance_with_enchanted_bonus',enchantment='minecraft:looting',unenchanted_chance=chance,enchanted_chance={'type':'minecraft:linear','base':chance+mult,'per_level_above_first':mult})
  if v.get('function')=='minecraft:enchant_randomly'and'enchantments'in v:v['options']=v.pop('enchantments')
  if isinstance(v.get('enchantments'),list)and not v.get('function'):
   entries=v.pop('enchantments')
   for e in entries:
    if 'enchantment'in e:e['enchantments']=e.pop('enchantment')
   v.setdefault('predicates',{})['minecraft:enchantments']=entries
  if isinstance(v.get('tag'),str):v['items']='#'+v.pop('tag')
  if 'structure'in v and isinstance(v['structure'],str):v['structures']=v.pop('structure')
  if 'items'in v and isinstance(v['items'],list)and len(v['items'])==1 and isinstance(v['items'][0],str):v['items']=v['items'][0]
  if 'function'in v:v['type']=v.pop('function')
  if 'condition'in v and'type'not in v:v['type']=v.pop('condition')
  if 'functions'in v:v['modifier']=v.pop('functions')
  return v
 for folder in ['loot_table','advancement']:
  for p in (ROOT/'data/arcadiandream'/folder).rglob('*.json'):
   d=loot(json.loads(p.read_text()))
   if folder=='loot_table'and d.get('type')=='minecraft:entity':d.setdefault('random_sequence','arcadiandream:'+p.relative_to(ROOT/'data/arcadiandream/loot_table').with_suffix('').as_posix())
   if folder=='advancement':
    for c in d.get('criteria',{}).values():
     cond=c.get('conditions',{})
     if c.get('trigger')=='minecraft:inventory_changed'and isinstance(cond.get('items'),dict):cond['items']=[cond['items']]
    if p.stem=='mochi_mallet_kills':d['criteria']={'kills_accumulated':{'trigger':'arcadiandream:mochi_mallet_kills','conditions':{'kills':100}}}
    if p.stem=='curtain_fire':d['criteria']={'shot_danmaku':{'trigger':'arcadiandream:curtain_fire','conditions':{'density':25}}}
   write(p,d)
 for name in ['ritual_shrine','danmaku_crafting_table','mysterious_seal']:
  p=ROOT/f'data/arcadiandream/horizon/block/{name}.json';d=json.loads(p.read_text())
  if name=='ritual_shrine':d['template']='minecraft:amethyst_block'
  elif name=='danmaku_crafting_table':d['template']='minecraft:chest';d.pop('state_properties',None);d['default_state']={'type':'single','waterlogged':'false'};d['shapes']={'property':'facing','values':{n:[[0,0,0,1,1,1]]for n in ['north','south','east','west']}}
  else:d['template']='minecraft:torch';d['state_properties']={'facing':{'type':'enum','values':['north','south','east','west','up','down']},'waterlogged':{'type':'boolean'}};d['default_state']={'facing':'north','waterlogged':'false'}
  write(p,d)
 for name,item in [('iron_ingots','iron_ingot'),('gold_ingots','gold_ingot'),('netherite_ingots','netherite_ingot'),('diamonds','diamond')]:
  write(ROOT/f'data/c/tags/item/{name}.json',{'replace':False,'values':['minecraft:'+item]})
 ids=json.loads((ROOT/'upstream.json').read_text())['items']
 for name in ids:
  d={'model':{'type':'minecraft:model','model':'arcadiandream:item/'+name}}
  if name.endswith('_shot'):d['model']['tints']=[{'type':'minecraft:constant','value':0xffffff},{'type':'minecraft:dye','default':0xff0000}]
  write(ROOT/f'assets/arcadiandream/items/{name}.json',d)
 for p in (ROOT/'data/arcadiandream/advancement').rglob('*.json'):
  d=json.loads(p.read_text());icon=d.get('display',{}).get('icon',{})
  if 'item'in icon:icon['id']=icon.pop('item')
  write(p,d)
 # Two music discs use the current jukebox song registry.
 for name,length,signal in [('fairy_playground',180,10),('the_shrine_long_forgotten',134,14)]:
  song={'sound_event':'arcadiandream:music_disc.'+name,'description':{'translate':'item.arcadiandream.music_disc_'+name+'.desc'},'length_in_seconds':length,'comparator_output':signal}
  write(ROOT/f'data/arcadiandream/jukebox_song/{name}.json',song)
  p=ROOT/f'data/arcadiandream/horizon/item/music_disc_{name}.json';d=json.loads(p.read_text());d['template']='minecraft:music_disc_13';d['components']['minecraft:jukebox_playable']={'song':'arcadiandream:'+name};write(p,d)
 # Keep vanilla tool interactions, but apply the original material/weapon combat values.
 for p in (ROOT/'data/arcadiandream/horizon/item').glob('*.json'):
  name=p.stem;d=json.loads(p.read_text());c=d['components'];attrs=[]
  if name.startswith(('makaite_','hihiirokane_')):
   material,kind=name.split('_',1);damage=2 if material=='makaite'else 5;speed=6 if material=='makaite'else 11
   if kind in ['sword','pickaxe','axe','shovel','hoe']:
    attack,rate={'sword':(3,-2.4),'pickaxe':(1,-2.8),'axe':(6,-3.1),'shovel':(1.5,-3),'hoe':(-2,-1)}[kind];attrs=[('attack_damage',damage+attack,'mainhand'),('attack_speed',rate,'mainhand')]
    if kind!='sword':c['minecraft:tool']={'rules':([{'blocks':'#minecraft:incorrect_for_iron_tool','correct_for_drops':False}]if material=='makaite'else[])+[{'blocks':'#minecraft:mineable/'+kind,'speed':speed,'correct_for_drops':True}],'default_mining_speed':1.0,'damage_per_block':1}
   if kind in ['helmet','chestplate','leggings','boots']:
    slot={'helmet':'head','chestplate':'chest','leggings':'legs','boots':'feet'}[kind];protection=({'helmet':2,'chestplate':5,'leggings':6,'boots':2}if material=='makaite'else{'helmet':4,'chestplate':7,'leggings':9,'boots':4})[kind];attrs=[('armor',protection,slot)]
    if material=='hihiirokane':attrs+=[('armor_toughness',4,slot),('knockback_resistance',0.1,slot)]
  special={'hisou_sword':(6,-2),'nue_trident':(9,-2.9),'mochi_mallet':(3,-3.2),'death_scythe':(6,-3),'miracle_mallet':(6,-3.2),'folding_chair':(3,-3.1),'icicle_sword':(4,-2.4)}
  if name=='miracle_mallet':c['minecraft:max_damage']=1561
  if name=='folding_chair':c['minecraft:max_damage']=200
  if name in special:damage,rate=special[name];attrs=[('attack_damage',damage,'mainhand'),('attack_speed',rate,'mainhand')]
  if name=='ordinary_hat':c['minecraft:equippable']={'slot':'head','asset_id':'arcadiandream:ordinary'};attrs=[('armor',1,'head')]
  if attrs:c['minecraft:attribute_modifiers']=[{'id':f'arcadiandream:{name}_{attr}','type':'minecraft:'+attr,'amount':amount,'operation':'add_value','slot':slot}for attr,amount,slot in attrs]
  if name=='hisou_sword':c['minecraft:repairable']={'items':'arcadiandream:dragon_gem'}
  if name=='mochi_mallet':c['minecraft:repairable']={'items':'minecraft:rabbit_hide'}
  if name in ['healing_charm','hourai_elixir','ibuki_gourd']:
   c['minecraft:consumable']={'consume_seconds':1.25 if name=='healing_charm'else 1.6,'animation':'bow'if name=='healing_charm'else'drink','has_consume_particles':False};c['minecraft:food']={'nutrition':0,'saturation':0,'can_always_eat':True}
  write(p,d)
 for p in list((ROOT/'data/arcadiandream/worldgen/configured_feature').glob('*.json'))+list((ROOT/'data/arcadiandream/worldgen/processor_list').glob('*.json')):
  d=json.loads(p.read_text())
  if isinstance(d.get('config'),dict):d.update(d.pop('config'))
  def states(v):
   if isinstance(v,dict):return {('id'if k=='Name'else'properties'if k=='Properties'else k):states(value)for k,value in v.items()}
   if isinstance(v,list):return [states(x)for x in v]
   return v
  write(p,states(d))
 for family in ['circle','bubble','amulet','star','kunai','pellet']:
  for n,scale in enumerate([1,1.75,2.5,3.25,4]):
   for phase in range(16 if family=='star'else 1):
    name=f'arcadian_projectile/{family}_{n}_{phase}'
    write(ROOT/f'assets/arcadiandream/models/{name}.json',{'parent':f'arcadiandream:item/{family}_shot','display':{'ground':{'rotation':[0,0,phase*22.5],'translation':[0,0,0],'scale':[0.5*scale]*3}}})
    write(ROOT/f'assets/arcadiandream/items/{name}.json',{'model':{'type':'minecraft:model','model':'arcadiandream:'+name,'tints':[{'type':'minecraft:constant','value':0xffffff},{'type':'minecraft:dye','default':0xff0000}]}})
 for stage in range(11):write(ROOT/f'assets/arcadiandream/items/mochi_mallet_{stage}.json',{'model':{'type':'minecraft:model','model':f'arcadiandream:item/mochi_mallet_{stage}'}})
 # Fabric's commented translation files contain arrays; Horizon's map loader requires strings.
 for p in (ROOT/'assets/arcadiandream/lang').glob('*.json'):
  language={k:v for k,v in json.loads(p.read_text()).items()if isinstance(v,str)}
  for key,value in list(language.items()):
   if key.startswith('item.arcadiandream.'):language.setdefault(key.replace('item.arcadiandream.','block.arcadiandream.',1),value)
  write(p,language)
 # Current registry-based enchantments, with original levels and costs.
 for name,maxlevel,cost,step,treasure in [('danmaku_protection',4,10,8,True),('youkai_buster',5,5,8,False)]:
  d={'description':{'translate':'enchantment.arcadiandream.'+name},'supported_items':'#minecraft:enchantable/armor'if name=='danmaku_protection'else'#minecraft:enchantable/weapon','weight':2,'max_level':maxlevel,'min_cost':{'base':cost,'per_level_above_first':step},'max_cost':{'base':cost+20,'per_level_above_first':step},'anvil_cost':4,'slots':['armor']if name=='danmaku_protection'else['mainhand'],'effects':{}}
  if name=='youkai_buster':
   d['exclusive_set']='#minecraft:exclusive_set/damage'
   d['effects']={'minecraft:damage':[{'effect':{'type':'minecraft:add','value':{'type':'minecraft:linear','base':2.5,'per_level_above_first':2.5}},'requirements':{'type':'minecraft:entity_properties','entity':'this','predicate':{'minecraft:entity_type':'#arcadiandream:fairies'}}}]}
  else:
   d['min_cost']={'base':12,'per_level_above_first':12};d['max_cost']={'base':36,'per_level_above_first':12};d['exclusive_set']='#minecraft:exclusive_set/armor'
   d['effects']={'minecraft:damage_protection':[{'effect':{'type':'minecraft:add','value':{'type':'minecraft:linear','base':2,'per_level_above_first':2}},'requirements':{'type':'minecraft:damage_source_properties','predicate':{'tags':[{'id':'#arcadiandream:is_danmaku','expected':True}]}}}]}
  write(ROOT/f'data/arcadiandream/enchantment/{name}.json',d)
  write(ROOT/f'data/minecraft/tags/enchantment/exclusive_set/{"damage"if name=="youkai_buster"else"armor"}.json',{'replace':False,'values':['arcadiandream:'+name]})
  if treasure:write(ROOT/'data/minecraft/tags/enchantment/treasure.json',{'replace':False,'values':['arcadiandream:'+name]})
 for dim,biomes in [('is_nether',['nether_wastes','soul_sand_valley','crimson_forest','warped_forest','basalt_deltas']),('is_end',['the_end','end_highlands','end_midlands','small_end_islands','end_barrens'])]:
  write(ROOT/f'data/minecraft/tags/worldgen/biome/{dim}.json',{'replace':False,'values':['minecraft:'+b for b in biomes]})
 for material in ['wooden','stone','iron','diamond']:
  write(ROOT/f'data/minecraft/tags/block/incorrect_for_{material}_tool.json',{'replace':False,'values':['arcadiandream:hihiirokane_ore','arcadiandream:deepslate_hihiirokane_ore']})
 write(ROOT/'data/arcadiandream/painting_variant/reimaden.json',{'asset_id':'arcadiandream:reimaden','width':1,'height':1,'title':{'translate':'painting.arcadiandream.reimaden.title'},'author':{'translate':'painting.arcadiandream.reimaden.author'}})
 write(ROOT/'data/minecraft/tags/painting_variant/placeable.json',{'replace':False,'values':['arcadiandream:reimaden']})
 write(ROOT/'data/arcadiandream/horizon/mob_effect/ethereal.json',{'category':'beneficial','color':0x96f1ff})
 write(ROOT/'data/arcadiandream/horizon/mob_effect/elixir_fatigue.json',{'category':'harmful','color':0xeb4034})
 print('Updated 26.3 item models, recipes, tags, equipment, songs and enchantments')
if __name__=='__main__':main()
