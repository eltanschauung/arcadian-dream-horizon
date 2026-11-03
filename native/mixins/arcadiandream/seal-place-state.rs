fn hook(level:&crate::game::GameLevel<'_>,state:BlockStateId,context:&PlacementContext,fluid:FluidState,answer:&mut Option<Option<BlockStateId>>){
 if level.native_state_block(state)!=Some("arcadiandream:mysterious_seal"){return}
 let facing=match context.face {Direction::Up=>"up",Direction::Down=>"down",Direction::East=>"east",Direction::West=>"west",Direction::South=>"south",Direction::North=>"north"};
 *answer=Some(level.native_state_with_property(state,"facing",facing).and_then(|s|level.native_state_with_property(s,"waterlogged",if fluid.kind()==FluidKind::Water{"true"}else{"false"})));
}
