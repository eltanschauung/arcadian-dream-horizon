extern crate horizon_client_lib as client;
use client::{assets::Assets,biome::Biomes,mod_content::{self,ContentMod},streaming::ChunkGenerator};
use horizon_world::worldgen::{generator::{GeneratorData,OverworldGenerator},overrides::WorldgenOverrides};
use std::path::Path;
fn main()->client::Result<()>{
 let a:Vec<_>=std::env::args().skip(1).collect();let mut mods=Vec::new();for file in &a[3..]{let p=Path::new(file);let package=horizon_mods::Package::read(p)?;mods.push(ContentMod{id:package.manifest.id.clone(),content:horizon_mods::Content::read(p,&package.manifest)?});}
 let root=mod_content::resource_root(Path::new(&a[0]),&mods,Path::new(&a[1]))?;let assets=Assets::load(&root)?;let biomes=Biomes::load(&root)?;
 let mut ov=WorldgenOverrides::default();ov.set_config_root(Path::new(&a[1]).join("config"));ov.set_mods([("tectonic_horizon".into(),a[2]=="true")].into());ov.activate();
 let data=GeneratorData::load_with(&root.join("worldgen"),assets.registry,&ov)?;println!("worldgen warnings: {:?}",data.warnings());
 for dim in ["minecraft:overworld","minecraft:the_nether","minecraft:the_end"]{
  let generator=OverworldGenerator::for_dimension(data.clone(),8448363262698333288,"minecraft:normal",dim)?;
  if dim=="minecraft:overworld" {
    let mut found=false;
    for(cx,cz)in [(0,0),(8,8),(24,-16),(-32,24),(64,64),(-64,-64),(0,100),(100,0)] {
     if let Some(start)=generator.worker().structure_start("arcadiandream:abandoned_shrine",horizon_core::position::ChunkPos::new(cx,cz)){assert!(!start.pieces.is_empty());println!("shrine structure: {} pieces at {cx},{cz}",start.pieces.len());found=true;break;}
    }
    assert!(found,"abandoned shrine failed to create any structure pieces");
   }
   let unsupported=generator.unsupported_features();println!("{dim}: unsupported {unsupported:?}");assert!(!unsupported.iter().any(|s|s.0.starts_with("arcadiandream:")),"Arcadian feature skipped");
  let terrain=client::vanillagen::VanillaTerrain::for_world_with(&root.join("worldgen"),&ov,assets.registry,&biomes,8448363262698333288,"minecraft:normal",dim,true)?;
  let mut counts=std::collections::BTreeMap::<String,usize>::new();let mut worker=terrain.worker();
  for (cx,cz)in [(0,0),(8,8),(24,-16),(-32,24)]{
   let mut region=std::array::from_fn::<_,9,_>(|i|terrain.proto(&mut worker,cx+i as i32%3-1,cz+i as i32/3-1));terrain.decorate(&mut worker,&mut region);let [_,_,_,_,center,..]=region;let chunk=terrain.finish(center);
   for y in chunk.min_y()..chunk.max_y(){for z in 0..16{for x in 0..16{let id=chunk.get(x,y,z);if let Some(s)=assets.registry.state(horizon_world::registry::BlockStateId(id)){if s.name.starts_with("arcadiandream:"){*counts.entry(s.name.clone()).or_default()+=1;}}}}}
  }
  println!("{dim}: ores {counts:?}");assert!(!counts.is_empty(),"no Arcadian ores generated in {dim}");
 }Ok(())
}
