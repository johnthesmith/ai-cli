/*
    SPDX-License-Identifier: MIT
    SPDX-FileCopyrightText: 2026 Still Swamp
*/

/*
    Chat section
    It contains methods prompt
*/

impl Ai
{
    /*
        Return file for current chat id
    */
    fn get_chat_file( &self )
    -> String
    {
        core::expand_path
        (
            &self.get_config_str
            (
                &[ "chat-file" ],
                "%profile-path%/chat.txt"
            )
            .replace( "%profile-path%", &self.get_profile_path())
            .replace( "%profile%", &self.get_profile() )
        )
    }



    fn get_chats_path( &self )
    -> String
    {
        core::expand_path
        (
            &self.get_config_str
            (
                &[ "chats-path" ],
                "%profile-path%/chats"
            )
            .replace( "%profile-path%", &self.get_profile_path())
        )
    }



    fn get_chat_path
    (
        &self,
        chat_id: &str
    )
    -> String
    {
        core::expand_path
        (
            &self.get_config_str
            (
                &[ "chat-path" ],
                "%chats-path%/%chat%"
            )
            .replace( "%profile-path%", &self.get_profile_path())
            .replace( "%chats-path%", &self.get_chats_path())
            .replace( "%chat%", chat_id )
        )
    }



    /*
        Return list of chats
    */
    fn get_chats( &self )
    -> Vec<String>
    {
        let mut result = Vec::new();
        result.push("default".to_string());
        let path = self.get_chats_path();
        if let Ok(entries) = std::fs::read_dir(&path)
        {
            for entry in entries.flatten()
            {
                if let Ok(file_type) = entry.file_type()
                {
                    if file_type.is_dir()
                    {
                        if let Some(name) = entry.file_name().to_str()
                        {
                            let chat_name = name.to_string();
                            if !result.contains(&chat_name)
                            {
                                result.push(chat_name);
                            }
                        }
                    }
                }
            }
        }
        result
    }



    /*
        Return max length for chat prompt
    */
    fn get_max_prompt_bytes( &self )
    -> usize
    {
        self.get_config_size( &[ "max-prompt-bytes" ], 100000 )
    }




    fn read_chat( &self )
    -> String
    {
        let path = self.get_chat_file();

        if let Ok( content ) = std::fs::read_to_string( &path )
        {
            let id = content.trim().to_string();
            if !id.is_empty()
            {
                return id;
            }
        }
        "default".to_string()
    }



    /*
        Change current chat id
    */
    fn bind_chat
    (
        &mut self,
        new_id: &str
    )
    -> &mut Self
    {
        let file_path = self.get_chat_file();

        if let Err(e) = core::ensure_directory( &file_path )
        {
            self.app.get_log_mut()
            .error( "Failed to ensure chat directory" )
            .prm( "error", &e);
            return self;
        }

        if let Err(e) = std::fs::write(&file_path, new_id)
        {
            self.app.get_log_mut()
            .error( "Failed to bind chat" )
            .prm( "path", &file_path)
            .prm( "error", &e.to_string());
        }
        else
        {
            self.app.get_log_mut()
            .trace( "Chat binded" )
            .prm( "id", new_id);
        }

        self
    }



    /*
        Return chat for current session
    */
    fn get_chat( &self )
    -> String
    {
        self.chat.clone()
    }



    /*
        Set chat for current session
    */
    fn set_chat
    (
        &mut self,
        name: &str
    )
    -> &mut Self
    {
        self.chat = name.to_string();
        self
    }
}
