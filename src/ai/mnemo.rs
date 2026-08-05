/*
    SPDX-License-Identifier: MIT
    SPDX-FileCopyrightText: 2026 Still Swamp
*/

impl Ai
{
    /*
        Colorize and push mnemo of action
    */
    fn mnemo
    (
        &mut self,
        text: &str
    )
    {
        let mut color = Color::White;
        if text.contains( '+' )
        {
            color = Color::Cyan;
        }

        if text.contains( '^' )
        {
            color = Color::Magenta;
        }

        if text.contains( '-' )
        {
            color = Color::Yellow;
        }

        if text.contains( '!' )
        {
            color = Color::Red;
        }

        self.status.push
        (
            Color::colorize( color, text, Color::Default, self.colorize )
        );
    }
}
