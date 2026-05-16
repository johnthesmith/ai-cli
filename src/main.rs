mod ai;

use crate::ai::Ai;

fn main() 
{
    let mut ai = Ai::create();
    ai.run();
}



//
//    let mut log = Log::create();
//    log
//    .begin( "application" )
//    .info( "info" )
//    .error( "error" )
//    .text( "uku" )
//    .warning( "warning" )
//    .text( "molagmar" )
//    .debug( "debug" )
//    .begin( "subsection" )
//    .trace( "trace" )
//    .prm( "a", "b" )
//    .prm( "b", 12 )
//    .prm( "c", -12.5 )
//    .text( "mimimi" )
//    .end( "" )
//    .end( "end of application" )
//    .eol();
