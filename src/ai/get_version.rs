/*
    SPDX-License-Identifier: MIT
    SPDX-FileCopyrightText: 2026 Still Swamp
*/

impl Ai
{
    /*
        Return tool version
    */
    fn get_version( &self )
    -> String
    {
        format!( "AI CLI Utility v{}", env!( "CARGO_PKG_VERSION" ))
    }
}
