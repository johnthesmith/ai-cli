/*
    SPDX-License-Identifier: MIT
    SPDX-FileCopyrightText: 2026 Still Swamp
*/

/*
        History section
*/

impl Ai
{
    fn get_history_file_path( &self )
    -> String
    {
        core::expand_path
        (
            &self.get_config_str
            (
                &[ "history" ],
                "%chat-path%/history.txt"
            )
            .replace( "%chat-path%", &self.get_chat_path( &self.get_chat() ))
            .replace( "%profile-path%", &self.get_profile_path() )
            .replace( "%profile%", &self.get_profile() )
            .replace( "%model%", &self.get_model_safe() )
            .replace( "%provider%", &self.get_provider() )
            .replace( "%chat%", &self.get_chat() )
        )
    }



    /*
        Return history file size
    */
    fn get_history_file_size( &self )
    -> u64
    {
        let path = self.get_history_file_path();
        std::fs::metadata(&path)
        .map(|m| m.len())
        .unwrap_or(0)
    }



    /*
        Clear history file
    */
    fn clear_history(&mut self) -> &mut Self
    {
        let history_path = self.get_history_file_path();
        if let Err( e ) = std::fs::remove_file( &history_path )
        {
            if e.kind() != std::io::ErrorKind::NotFound
            {
                self.app.get_log_mut()
                .error( "Failed to clear history" )
                .prm( "path", &history_path )
                .prm( "error", &e.to_string() );
            }
        }
        else
        {
            self.app.get_log_mut()
            .info( "History cleared" )
            .prm( "path", &history_path );
        }
        self
    }



    /*
        Read history content
    */
    fn read_history_content(&mut self)
    -> String
    {
        let path = self.get_history_file_path();

        match std::fs::read_to_string(&path)
        {
            Ok(content) => content,

            Err(e) if e.kind() == std::io::ErrorKind::NotFound =>
            {
                String::new()
            }

            Err(e) =>
            {
                self.app.get_log_mut()
                .error("Failed to read history file")
                .prm("path", &path)
                .prm("error", &e.to_string());

                String::new()
            }
        }
    }


    /*
        Send history to stdout
    */
    fn out_history(&mut self)
    -> &mut Self
    {
        let history = self.read_history_content();

        if history.is_empty()
        {
            println!("No history");
        }
        else
        {
            println!("{}", history);
        }

        self
    }
}
