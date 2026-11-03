extern crate horizon_client_lib as client;
use client::{assets::Assets,blocks::BlockFacts,biome::Biomes,items::Items,crafting::Recipes,mod_content::{self,ContentMod}};
use horizon_world::loot::PackOverlay;
use std::path::Path;
fn main()->client::Result<()>{
 let a:Vec<_>=std::env::args().skip(1).collect();let mut mods=Vec::new();
 for file in &a[2..]{let p=Path::new(file);let package=horizon_mods::Package::read(p)?;mods.push(ContentMod{id:package.manifest.id.clone(),content:horizon_mods::Content::read(p,&package.manifest)?});}
 let root=mod_content::resource_root(Path::new(&a[0]),&mods,Path::new(&a[1]))?;
 let assets=Assets::load(&root)?;let facts=BlockFacts::load(&root,assets.registry)?;let items=Items::load(&root,assets.registry,&facts,&PackOverlay::NONE)?;
 let recipes=Recipes::load(&root,&items,&PackOverlay::NONE)?;let biomes=Biomes::load(&root)?;
 for id in ["circle_shot","fairy_spawn_egg","mysterious_seal","hihiirokane_sword","ritual_shrine"]{assert!(items.get(&format!("arcadiandream:{id}")).is_some(),"missing item {id}");}
 let mut random=horizon_core::random::LegacyRandom::new(1234);
 let ore=assets.registry.default_state("arcadiandream:hihiirokane_ore").unwrap();
 let mut pick=client::items::ItemStack::new("minecraft:netherite_pickaxe",1);
 let drops=items.block_drops(assets.registry,ore,Some(&pick),&mut random);assert!(!drops.is_empty()&&drops.iter().all(|s|s.item.as_ref()=="arcadiandream:hihiirokane_chunk"),"normal ore drops {drops:?}");
 pick.data_mut().enchantments.push(("minecraft:silk_touch".into(),1));
 assert_eq!(pick.enchantment_level("minecraft:silk_touch"),1);let drops=items.block_drops(assets.registry,ore,Some(&pick),&mut random);assert_eq!(drops.len(),1);assert_eq!(drops[0].item.as_ref(),"arcadiandream:hihiirokane_ore");
 let language=client::lang::Language::load(&[&root],"en_us");client::lang::set(language);assert_eq!(client::lang::get("block.arcadiandream.circle_shot"),"Circle Shot");assert_eq!(client::lang::get("block.arcadiandream.danmaku_crafting_table"),"Danmaku Crafting Table");
 let tables=horizon_world::loot::shared_tables(&root,&PackOverlay::NONE)?;
 assert!(tables["minecraft:chests/end_city_treasure"]["pools"].as_array().unwrap().iter().any(|p|p["entries"][0]["name"]=="arcadiandream:dragon_gem"));
 for name in ["fairy","sunflower_fairy","ice_fairy"]{assert!(tables[&format!("arcadiandream:entities/{name}")]["random_sequence"].is_string());}
 println!("PASS content root {}: {} biomes, {} states; items and recipes loaded",root.display(),biomes.list.len(),assets.registry.len());let _=recipes;
 std::fs::write(Path::new(&a[1]).join("root.txt"),root.to_string_lossy().as_bytes())?;Ok(())
}
