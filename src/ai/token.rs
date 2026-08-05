/*
    SPDX-License-Identifier: MIT
    SPDX-FileCopyrightText: 2026 Still Swamp
*/

/*
    Token
*/

impl Ai
{
    /*
        Return token path for current provider
    */
    fn get_token_path( &self ) -> String
    {
        core::expand_path
        (
            &self.get_config_str
            (
                &[ "token" ],
                "~/.config/ai/app/cli/%profile%/tokens/%provider%.txt"
            )
            .replace( "%profile-path%", &self.get_profile_path() )
            .replace( "%profile%", &self.get_profile())
            .replace( "%chat%", &self.get_chat())
            .replace( "%provider%", &self.get_provider())
            .replace( "%model%", &self.get_model_safe())

        )
    }
}
