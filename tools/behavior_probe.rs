extern crate horizon_client_lib as client;
use client::{projectiles::{self,Projectile,ProjectileKind,ProjectileWorld,Hit,arcadian_motion as motion},mobs::Who};
use horizon_core::{geometry::{Vec3,Aabb},random::LegacyRandom};
use serde_json::json;
struct Scene(Vec<(Who,Aabb)>);
impl ProjectileWorld for Scene{
 fn clip(&self,_:Vec3,_:Vec3)->Option<(Vec3,[i32;3],[i32;3])>{None}
 fn in_water(&self,_:Aabb)->bool{false}
 fn targets(&self)->&[(Who,Aabb)]{&self.0}
}
fn main(){
 projectiles::register_thrown_types(["circle","bubble","amulet","star","kunai","pellet"].map(|f|(format!("arcadiandream:{f}_bullet"),format!("arcadiandream:{f}_shot"),2.0)));
 let base=projectiles::thrown_type("arcadiandream:circle_bullet").unwrap();
 let mut p=Projectile::new(ProjectileKind::Thrown(motion::damage_type(base,7.25)),Vec3::new(1.0,70.0,2.0),Some(Who::Player));p.id=10;p.velocity=Vec3::new(0.0,0.0,1.0);p.age=7;p.no_gravity=true;
 motion::register(p.id,json!({"power":7.25,"duration":40,"color":65535,"icy":true,"scale":2.5,"gravity":0.0}));
 let nbt=client::entity_nbt::throwable_to_nbt(&p,[1,2,3,4],None,None).unwrap();
 let mut restored=client::entity_nbt::throwable_from_nbt(&nbt,None).unwrap();restored.id=20;motion::prepare(&mut restored);
 assert_eq!(restored.age,7);assert_eq!(motion::data(restored.id)["color"],65535);assert_eq!(motion::data(restored.id)["icy"],true);assert!(matches!(restored.kind,ProjectileKind::Thrown(t)if (t.damage-7.25).abs()<0.001));
 let drawn=restored.drawn_item().unwrap();assert_eq!(drawn.data().unwrap().dyed_color,Some(65535));
 let scene=Scene(vec![]);for _ in 0..33{assert!(motion::tick(&mut restored,&scene,true).unwrap().0)}assert!(!motion::tick(&mut restored,&scene,true).unwrap().0);
 let kunai=projectiles::thrown_type("arcadiandream:kunai_bullet").unwrap();let mut p=Projectile::new(ProjectileKind::Thrown(kunai),Vec3::new(0.0,70.0,0.0),None);p.id=30;p.velocity=Vec3::new(0.0,0.0,2.0);motion::register(30,json!({"duration":100}));
 let scene=Scene(vec![(Who::Mob(50),Aabb::new(Vec3::new(-0.5,69.0,1.0),Vec3::new(0.5,72.0,1.5)))]);
 let (alive,hit)=motion::tick(&mut p,&scene,true).unwrap();assert!(alive&&matches!(hit,Hit::Entity{who:Who::Mob(50),..}));assert!(matches!(motion::tick(&mut p,&scene,true).unwrap().1,Hit::None));
 let mut r=LegacyRandom::new(1234);let mut s=client::merchant::Session::new(client::merchant::arcadian_trades::level(1,&mut r),true);assert_eq!(s.offers.len(),2);
 s.level=1;s.xp=9;client::merchant::arcadian_trades::level_up(&mut s,0);assert_eq!(s.offers.len(),3);assert_eq!(s.offers[2].result.item.as_ref(),"arcadiandream:faith_item");
 println!("PASS: projectile power/color/icy/scale/age save roundtrip; duration; kunai piercing; Antiquarian level-up offers");
}
