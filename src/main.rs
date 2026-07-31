/*
    SPDX-License-Identifier: MIT
    SPDX-FileCopyrightText: 2026 Still Swamp
*/

/*
    Main entry point for AI CLI utility.

    Creates and runs the AI assistant instance.
    Handles command-line arguments, configuration, and TTY input/output.
*/

mod ai;

use crate::ai::Ai;



fn main()
{
    let mut ai = Ai::create();
    ai.run();
}

