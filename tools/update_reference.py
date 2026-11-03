#!/usr/bin/env python3
"""Refresh numeric reference data from an explicitly supplied original checkout."""
from pathlib import Path
import json,re,sys
ROOT=Path(__file__).resolve().parents[1]
S=Path(sys.argv[1])/'src/main/java/net/reimaden/arcadiandream'
def main():
 text=(S/'item/ModItems.java').read_text();shots={}
 for m in re.finditer(r'registerItem\("([a-z]+)_shot",\s*new .*?maxDamage\((\d+)\),\s*([^;]+)\);',text,re.S):
  args=[v.strip().strip('"').rstrip('f') for v in m[3].rstrip(')').split(',')];keys=['power','speed','duration','cooldown','gravity','divergence','pattern','density','maxPower','maxSpeed','maxDuration','maxCooldown','maxDivergence','maxDensity'];d={k:(v if k=='pattern' else float(v)) for k,v in zip(keys,args)};d['durability']=int(m[2]);shots[m[1]]=d
  item=ROOT/f'data/arcadiandream/horizon/item/{m[1]}_shot.json';data=json.loads(item.read_text());data['components']['minecraft:max_damage']=int(m[2]);data['components'].pop('minecraft:custom_data',None);item.write_text(json.dumps(data,indent=2)+'\n')
 attacks={}
 for name,file,cooldowns in [('fairy','FairyEntity',[20,40,25,30,40,40]),('sunflower_fairy','SunflowerFairyEntity',[20,40,60,30,12,35,25,20]),('ice_fairy','IceFairyEntity',[6,20,20,10,12,40,25,30])]:
  text=(S/f'entity/custom/mob/{file}.java').read_text();rows=[]
  for m in re.finditer(r'private void (\w+)\([^)]*\) \{\s*create(\w+)\((.*)\);',text):
   args=[v.strip().rstrip('f')for v in m[3].split(',')];args=args[2:]if m[2]=='Rain'else args[3:]
   row={'name':m[1],'pattern':m[2].lower(),'args':args,'cooldown':cooldowns[len(rows)]};rows.append(row)
  assert len(rows)==len(cooldowns),(name,rows)
  attacks[name]=rows
 (ROOT/'reference/behavior.json').write_text(json.dumps({'shots':shots,'fairy_attacks':attacks},indent=2)+'\n')
if __name__=='__main__':main()
