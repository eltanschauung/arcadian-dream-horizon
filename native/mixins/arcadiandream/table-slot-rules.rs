fn hook(menu:&str,slot:usize,stack:&ItemStack,may_place:&mut bool,max_stack:&mut u32){
    if menu!="arcadiandream:danmaku_crafting_table"||slot>=7{return}
    let item=stack.item.as_ref();*may_place=match slot{
        0=>item.starts_with("arcadiandream:")&&item.ends_with("_bullet_core"),
        1=>item.starts_with("arcadiandream:")&&item.ends_with("_shot"),2=>false,
        3=>["arcadiandream:power_item","arcadiandream:big_power_item","arcadiandream:point_item","arcadiandream:star_item","arcadiandream:time_orb","arcadiandream:enchanted_ice"].contains(&item),
        4=>item=="arcadiandream:faith_item",5=>item.starts_with("arcadiandream:")&&item.ends_with("_pattern"),
        6=>item.starts_with("minecraft:")&&item.ends_with("_dye"),_=>false};if slot==1{*max_stack=1;}
}
