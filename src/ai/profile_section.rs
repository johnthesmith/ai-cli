/*
    SPDX-License-Identifier: MIT
    SPDX-FileCopyrightText: 2026 Still Swamp
*/

/*
    Profile section
*/

impl Ai
{
    fn get_profiles_path( &self )
    -> String
    {
        let mut current = match std::env::current_dir()
        {
            Ok(dir) => dir,
            Err(_) => return String::new(),
        };
        loop
        {
            let ai_path = current.join( AI_FOLDER );
            if ai_path.exists() && ai_path.is_dir()
            {
                return ai_path.display().to_string();
            }
            if !current.pop()
            {
                break;
            }
        }
        String::new()
    }



    /*
        Return profile file
    */
    fn get_profile_file( &self )
    -> String
    {
        core::expand_path( &(self.get_profiles_path() + "/profile.txt" ))
    }




    fn get_profile_path( &self )
    -> String
    {
        core::expand_path
        (
            &(self.get_profiles_path() + "/profiles/" + &self.profile )
        )
    }



    /*
        Set profile
    */
    fn set_profile
    (
        &mut self,
        /* Profile name */
        name: &str
    )
    -> &mut Self
    {
        self.profile = name.to_string();
        self
    }



    /*
        Return profile
    */
    fn get_profile( &self )
    -> &str
    {
        &self.profile
    }



    /*
        Read and return profile
    */
    fn read_profile( &mut self )
    -> &mut Self
    {
        let path = self.get_profile_file();

        let profile = std::fs::read_to_string(&path)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "default".to_string());

        self.set_profile( &profile );
        self
    }



    /*
        Write profile in to file
    */
    fn write_profile
    (
        &mut self,
        name: &str
    ) -> &mut Self
    {
        let path = self.get_profile_file();

        if let Err(e) = std::fs::write( &path, name )
        {
            /* Set state for app */
            self.app.state.set_state
            (
                "PROFILE_WRITE_ERROR",
                json!
                (
                    {
                        "path": path,
                        "error": e.to_string()
                    }
                )
            );
            /* Write in to log */
            self.app.get_log_mut()
            .error( "Failed to write profile" )
            .prm( "path", &path)
            .prm( "error", &e.to_string());
        }
        else
        {
            self.app.get_log_mut()
            .trace( "Profile saved" )
            .prm( "name", name);
        }

        self
    }

}
