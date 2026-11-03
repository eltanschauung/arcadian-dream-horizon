#!/usr/bin/env python3
"""Focused Rust math/controller regression checks; no game or player home needed."""
from pathlib import Path
import subprocess
ROOT=Path(__file__).resolve().parents[1]
CHECK=ROOT/'.local/check';CHECK.mkdir(parents=True,exist_ok=True)
s=(ROOT/'native/mixins/arcadiandream/render.rs').read_text()
pure='use serde_json::Value;\n'+s[s.index('fn vector('):s.index("impl Game<'_>")]
tests=r'''
fn near(a:f32,b:f32){assert!((a-b).abs()<0.0001,"{a} != {b}");}
#[test] fn original_curves(){
 for (name,t,want) in [("easeInOutSine",0.25,0.1464466),("easeInOutCirc",0.25,0.0669873),("easeInCirc",0.5,0.1339746),("easeOutElastic",0.5,0.25)] {
  let v=serde_json::json!({"0.0":{"vector":[0,0,0]},"1.0":{"vector":[1,1,1],"easing":name,"easingArgs":[0.5]}});
  near(frame(&v,t,[0.0;3])[0],want);
 }
}
#[test] fn original_selection(){
 assert_eq!(select_clip(true,false,true),"move");
 assert_eq!(select_clip(true,true,true),"idle.ground");
 assert_eq!(select_clip(true,true,false),"idle");
 assert_eq!(select_clip(false,false,true),"idle.ground");
 assert_eq!(select_clip(false,false,false),"idle");
}
#[test] fn transitions_restart_and_loop(){
 let a=serde_json::json!({"idle":{"animation_length":1,"bones":{"body":{"position":[0,0,0]}}},"move":{"animation_length":1,"bones":{"body":{"position":{"0.0":[0,10,0],"1.0":[0,30,0]}}}}});
 let b=vec![serde_json::json!({"name":"body"})];let mut c=Controller::default();
 c.sample("idle",0.0,&a,&b);c.sample("idle",5.0,&a,&b);
 near(c.sample("move",10.0,&a,&b)["body"].position[1],0.0);
 near(c.sample("move",12.5,&a,&b)["body"].position[1],5.0);
 near(c.sample("move",15.0,&a,&b)["body"].position[1],10.0);
 near(c.sample("move",25.0,&a,&b)["body"].position[1],20.0);
 near(c.sample("move",35.0,&a,&b)["body"].position[1],10.0);
 near(c.sample("idle",35.0,&a,&b)["body"].position[1],10.0);
 near(c.sample("idle",37.5,&a,&b)["body"].position[1],5.0);
 near(c.sample("idle",40.0,&a,&b)["body"].position[1],0.0);
}
#[test] fn gecko_baked_axes(){
 assert_eq!(geo_position([-3.0,15.0,0.0]),[3.0,15.0,0.0]);
 let r=geo_rotation([127.0604,-51.5359,-132.7482]);
 near(r[0],-127.0604);near(r[1],51.5359);near(r[2],-132.7482);
}
'''
tests+=r'''
#[test] fn java_death_roll(){
 near(death_roll(0,0.0),0.0);near(death_roll(1,0.0),0.0);
 near(death_roll(6,0.0),56.920998);near(death_roll(20,0.0),90.0);
}
'''
source=CHECK/'fairy-animation-test.rs' ;source.write_text(pure+tests)
launcher=Path.home()/'.local/share/horizon-launcher';rust=next((launcher/'rust').glob('*/bin/rustc'))
deps=launcher/'fork-native/target/fast/deps';libs=list(deps.glob('libserde_json-*.rlib'))
for lib in libs:
 p=subprocess.run([str(rust),'--edition=2024','--test',str(source),'--crate-name','fairy_animation','-L',f'dependency={deps}','--extern',f'serde_json={lib}','-o',str(CHECK/'fairy-animation-test')],capture_output=True,text=True)
 if p.returncode==0:break
else:raise RuntimeError(p.stderr)
subprocess.run([str(CHECK/'fairy-animation-test')],check=True)
