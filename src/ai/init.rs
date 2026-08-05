/*
    SPDX-License-Identifier: MIT
    SPDX-FileCopyrightText: 2026 Still Swamp
*/

impl Ai
{
    /*
        Ai init implementation
    */
    fn init( &mut self )
    -> &mut Self
    {
        let path = self.get_profiles_path();
        if !path.is_empty()
        {
            self.app.state.set_state
            (
                "profile-already-exists",
                json!({ "path": path })
            );
        }
        else
        {
            let current = std::env::current_dir().unwrap();
            let ai_folder = current.join( AI_FOLDER );

            std::fs::create_dir_all(&ai_folder).unwrap();

            self.profile
            = current.file_name().unwrap().to_string_lossy().to_string();
        }

        self
    }
}
