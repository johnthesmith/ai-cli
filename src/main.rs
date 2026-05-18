mod ai;

use crate::ai::Ai;

fn main() 
{
    let mut ai = Ai::create();
    ai.run();
}
