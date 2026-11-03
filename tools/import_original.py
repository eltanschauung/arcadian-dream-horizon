#!/usr/bin/env python3
"""Import licensed Arcadian Dream 0.1.0 content; generate Horizon declarations."""
import argparse,hashlib,json,re,shutil
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def write(path,value):
 path.parent.mkdir(parents=True,exist_ok=True);path.write_text(json.dumps(value,indent=2,ensure_ascii=False)+'\n')
def main():
 ap=argparse.ArgumentParser();ap.add_argument('--source',type=Path,default=ROOT.parent/'arcadian-dream');a=ap.parse_args();src=a.source
 for name in ['LICENSE','LICENSE.ASSETS']:shutil.copy2(src/name,ROOT/name)
 assets=src/'unzipped_jar_assets/assets'
 shutil.copytree(assets,ROOT/'assets',dirs_exist_ok=True)
 shutil.copytree(src/'src/main/generated/resources/assets',ROOT/'assets',dirs_exist_ok=True)
 for base in [src/'src/main/resources/data',src/'src/main/generated/resources/data']:
  shutil.copytree(base,ROOT/'data',dirs_exist_ok=True)
 ns=ROOT/'data/arcadiandream/horizon';j=src/'src/main/java/net/reimaden/arcadiandream'
 blocks=re.findall(r'(?:registerBlock|registerWithoutItem)\("([a-z_]+)"', (j/'block/ModBlocks.java').read_text())
 specs={
 'onbashira':('minecraft:oak_planks',2,0), 'onbashira_pillar':('minecraft:oak_log',2,0),
 'ritual_shrine':('minecraft:amethyst_block',4,7),'danmaku_crafting_table':('minecraft:crafting_table',2.5,0),
 'mysterious_seal':('minecraft:glow_lichen',0.1,5),
 'dragon_gem_ore':('minecraft:diamond_ore',6,6),'deepslate_dragon_gem_ore':('minecraft:deepslate_diamond_ore',7.5,6),
 'end_stone_dragon_gem_ore':('minecraft:end_stone',12,6),'makaite_ore':('minecraft:nether_gold_ore',4,0),
 'hihiirokane_ore':('minecraft:diamond_ore',20,0),'deepslate_hihiirokane_ore':('minecraft:deepslate_diamond_ore',30,0),
 'raw_makaite_block':('minecraft:raw_iron_block',5,0),'hihiirokane_chunk_block':('minecraft:raw_iron_block',40,0),
 'dragon_gem_block':('minecraft:diamond_block',5,9),'makaite_block':('minecraft:iron_block',5,0),'hihiirokane_block':('minecraft:netherite_block',50,0)}
 for name in blocks:
  template,hardness,light=specs[name];d={'template':template,'properties':{'hardness':hardness,'light':light},'item':True}
  if name in ['ritual_shrine','danmaku_crafting_table']:d['template']='minecraft:chest';d['state_properties']={'facing':{'type':'enum','values':['north','east','south','west']}}
  write(ns/'block'/f'{name}.json',d)
 text=(j/'item/ModItems.java').read_text();ids=[]
 for match in re.finditer(r'registerItem\("([a-z_]+)"\s*,',text):
  name=match[1];start=match.end();nextdecl=text.find('public static final Item',start);body=text[start:text.index(';',start)]
  template='minecraft:stick';components={'minecraft:item_name':{'translate':'item.arcadiandream.'+name},'!minecraft:cooking_fuel':{}};d={'template':template,'components':components}
  for material,tier,dur in [('makaite','iron',905),('hihiirokane','netherite',2561)]:
   if name.startswith(material+'_'):
    kind=name[len(material)+1:]
    if kind in ['sword','pickaxe','axe','shovel','hoe']:d['template']=f'minecraft:{tier}_{kind}';components['minecraft:max_damage']=dur;components['minecraft:repairable']={'items':f'arcadiandream:{material}_ingot'}
    elif kind in ['helmet','chestplate','leggings','boots']:
     d['template']=f'minecraft:{tier}_{kind}';mult=24 if material=='makaite' else 41;components['minecraft:max_damage']=dict(boots=13*mult,leggings=15*mult,chestplate=16*mult,helmet=11*mult)[kind]
     components['minecraft:equippable']={'slot':dict(boots='feet',leggings='legs',chestplate='chest',helmet='head')[kind],'asset_id':'arcadiandream:'+material}
  if 'maxCount(' in body:d['max_stack_size']=int(re.search(r'maxCount\((\d+)\)',body)[1])
  if (damage := re.search(r'maxDamage\((\d+)\)',body)):components['minecraft:max_damage']=int(damage[1]);d['max_stack_size']=1
  rarity=re.search(r'rarity\(Rarity\.([A-Z]+)\)',body)
  if rarity:d['rarity']=rarity[1].lower()
  if '.fireproof()' in body:components['minecraft:damage_resistant']={'types':'#minecraft:is_fire'}
  special={'hisou_sword':('diamond_sword',1796),'nue_trident':('netherite_sword',2031),'mochi_mallet':('wooden_sword',190),'death_scythe':('iron_sword',250),'miracle_mallet':('wooden_sword',2031),'folding_chair':('iron_sword',250),'icicle_sword':('stone_sword',150),'ordinary_hat':('leather_helmet',44)}
  if name in special:d['template']='minecraft:'+special[name][0];components['minecraft:max_damage']=special[name][1];d['max_stack_size']=1
  food={'heavenly_peach':(6,0.4,True),'lamprey':(3,0.3,False),'cooked_lamprey':(10,0.8,False)}
  if name in food:
   n,sat,always=food[name];d['template']='minecraft:apple';components['minecraft:food']={'nutrition':n,'saturation':2*n*sat,'can_always_eat':always}
  if name in ['enchanted_ice','magatama_necklace','hourai_elixir']:components['minecraft:enchantment_glint_override']=True
  if name.endswith('_shot'):d['max_stack_size']=1;components['minecraft:max_damage']=512;components['minecraft:custom_data']={'arcadiandream':{'pattern':'spread','density':1,'power':2.0,'speed':1.0,'duration':60,'gravity':0.0,'cooldown':10,'divergence':1.0,'color':0xffffff}}
  if name.endswith('_spawn_egg'):d['max_stack_size']=64
  if name.startswith('music_disc_'):d['max_stack_size']=1
  write(ns/'item'/f'{name}.json',d);ids.append(name)
 # One tab, containing every registered usable item and block.
 all_items=list(dict.fromkeys(ids+blocks))
 write(ns/'creative_tab/arcadian_dream.json',{'title':'Arcadian Dream','icon':'arcadiandream:dragon_gem','items':['arcadiandream:'+n for n in all_items]})
 for dim,features in [('overworld',['dragon_gem_ore','hihiirokane_ore','hihiirokane_ore_buried']),('the_nether',['makaite_ore']),('the_end',['end_dragon_gem_ore'])]:
  tag={'overworld':'is_overworld','the_nether':'is_nether','the_end':'is_end'}[dim]
  write(ROOT/f'data/arcadiandream/neoforge/biome_modifier/ores_{dim}.json',{'type':'neoforge:add_features','biomes':'#minecraft:'+tag,'features':['arcadiandream:'+f for f in features],'step':'underground_ores'})
 # The original forest convention tag is Fabric-specific; union current vanilla and BOP forests.
 write(ROOT/'data/arcadiandream/tags/worldgen/biome/has_structure/abandoned_shrine.json',{'replace':False,'values':['#minecraft:is_forest','#minecraft:is_taiga',{'id':'#c:forest','required':False}]})
 # Modern component names for copied armor layers.
 for material in ['makaite','hihiirokane','ordinary']:
  layers={}
  for layer,idx in [('humanoid',1),('humanoid_leggings',2)]:
   old=ROOT/f'assets/arcadiandream/textures/models/armor/{material}_layer_{idx}.png'
   if old.exists():
    dest=ROOT/f'assets/arcadiandream/textures/entity/equipment/{layer}/{material}.png';dest.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(old,dest);layers[layer]=[{'texture':'arcadiandream:'+material}]
  if layers:write(ROOT/f'assets/arcadiandream/equipment/{material}.json',{'layers':layers})
 # Java-loader integration data is retained as reference; native equivalents own runtime behavior.
 for name in ['assets/emi','assets/minecraft','data/trinkets','data/fabric']:
  p=ROOT/name
  if p.exists():
   dest=ROOT/'reference/compatibility'/name;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copytree(p,dest,dirs_exist_ok=True);shutil.rmtree(p)
 # Native-thrown projectile types, one per configurable shot.
 for name in ['circle','bubble','amulet','star','kunai','pellet']:
  write(ns/'entity_type'/f'{name}_bullet.json',{'template':'minecraft:snowball','item':f'arcadiandream:{name}_shot','damage':2.0})
 for name,color in [('ethereal',0xb5d9ff),('elixir_fatigue',0x806080)]:write(ns/'mob_effect'/f'{name}.json',{'category':'neutral','color':color})
 write(ns/'menu/ritual_shrine.json',{'slots':1,'data_slots':1,'results':[]})
 write(ns/'menu/danmaku_crafting_table.json',{'slots':7,'data_slots':1,'results':[2]})
 provenance={'repository':'https://github.com/Maxmani/arcadian-dream','revision':'46aef7b6c35790ccc4b7a9236aa25907cc743d94','release':'0.1.0','release_jar':'arcadiandream-1.20.1-0.1.0.jar','code_license':'EUPL-1.2-or-later','asset_license':'CC-BY-SA-4.0','assets_sha256':{str(p.relative_to(assets)):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(assets.rglob('*')) if p.is_file()},'items':all_items,'blocks':blocks}
 write(ROOT/'upstream.json',provenance)
 print(f'Imported {len(all_items)} items/block items, {len(blocks)} blocks and all assets/data')
if __name__=='__main__':main()
