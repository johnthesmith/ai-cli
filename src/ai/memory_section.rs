/*
    SPDX-License-Identifier: MIT
    SPDX-FileCopyrightText: 2026 Still Swamp
*/

/*
    Memory section
*/

impl Ai
{
    /*
        Return memory id for current session
    */
    fn get_memory_id( &self )
    -> String
    {
        self.memory_id.clone()
    }



    /*
        Return memory id for current session
    */
    fn set_memory_id
    (
        &mut self,
        cmd: &str
    )
    -> &mut Self
    {
        self.memory_id = cmd.to_string();
        self
    }



    fn get_memory_path( &self )
    -> String
    {
        core::expand_path
        (
            &self.get_config_str
            (
                &[ "memory-path" ],
                "%profile-path%/memory"
            )
            .replace( "%profile-path%", &self.get_profile_path())
            .replace( "%chat%", &self.get_chat())
        )
    }



    /*
        Return memory file
    */
    fn get_memory_file( &self )
    -> String
    {
        core::expand_path
        (
            &self.get_config_str
            (
                &[ "memory-file" ],
                "%memory-path%/%memory-id%.txt"
            )
            .replace( "%memory-path%", &self.get_memory_path() )
            .replace( "%profile-path%", &self.get_profile_path() )
            .replace( "%profile%", &self.get_profile() )
            .replace( "%provider%", &self.get_provider() )
            .replace( "%model%", &self.get_model_safe() )
            .replace( "%memory-id%", &self.get_memory_id() )
            .replace( "%chat%", &self.get_chat() )
        )
    }



    /*
        Return list of memories
    */
    fn get_memories(&self) -> Vec<String>
    {
        let mut result = Vec::new();
        result.push("default".to_string());

        let path = self.get_memory_path();

        if let Ok(entries) = std::fs::read_dir(&path)
        {
            for entry in entries.flatten()
            {
                if let Ok(file_type) = entry.file_type()
                {
                    if file_type.is_file()
                    {
                        if let Some(name) = entry.file_name().to_str()
                        {
                            if name.ends_with(".txt")
                            {
                                let id = name
                                .trim_end_matches(".txt")
                                .to_string();
                                if !result.contains(&id)
                                {
                                    result.push(id);
                                }
                            }
                        }
                    }
                }
            }
        }
        result
    }



    /*
        Return memory of file for current chat
    */
    fn get_memory_of_chat_file( &self )
    -> String
    {
        core::expand_path
        (
            &self.get_config_str
            (
                &[ "memory-of-chat-file" ],
                "%profile-path%/chats/%chat%/memory.txt"
            )
            .replace( "%profile-path%", &self.get_profile_path() )
            .replace( "%profile%", &self.get_profile() )
            .replace( "%provider%", &self.get_provider() )
            .replace( "%model%", &self.get_model_safe() )
            .replace( "%memory-id%", &self.get_memory_id() )
            .replace( "%chat%", &self.get_chat() )
        )
    }



    fn read_memory_of_chat( &self )
    -> String
    {
        let path = self.get_memory_of_chat_file();

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
        Bind memory for current chat
    */
    fn bind_memory
    (
        &mut self,
        id: &str
    )
    -> &mut Self
    {
        let file_path = self.get_memory_of_chat_file();

        if let Err(e) = core::ensure_directory( &file_path )
        {
            self.app.get_log_mut()
            .error( "Failed to ensure memory directory" )
            .prm( "error", &e );
            return self;
        }

        if let Err(e) = std::fs::write( &file_path, id )
        {
            self.app.get_log_mut()
            .error( "Failed to bind memory" )
            .prm( "path", &file_path )
            .prm( "error", &e.to_string() );
        }
        else
        {
            self.app.get_log_mut()
            .trace( "Memory binded" )
            .prm( "id", id );
        }

        self
    }



    /*
        Return memory file size
    */
    fn get_memory_file_size( &self )
    -> u64
    {
        let path = self.get_memory_file();
        std::fs::metadata(&path)
        .map(|m| m.len())
        .unwrap_or(0)
    }



    /*
        Clear memory file for current chat
    */
    fn clear_memory( &mut self )
    -> &mut Self
    {
        let path = self.get_memory_file();

        match std::fs::remove_file(&path)
        {
            Ok(_) =>
            {
                self.app.get_log_mut()
                .info("Memory file removed")
                .prm("path", &path);
            }
            Err(e) =>
            {
                if e.kind() == std::io::ErrorKind::NotFound
                {
                    self.app.get_log_mut()
                    .info("Memory file already removed")
                    .prm("path", &path);
                }
                else
                {
                    self.app.get_log_mut()
                    .error("Failed to remove memory file")
                    .prm("path", &path)
                    .prm("error", &e.to_string());
                }
            }
        }
        self
    }



    fn read_memory_content( &mut self )
    -> String
    {
        let path = self.get_memory_file();
        match std::fs::read_to_string(&path)
        {
            Ok(content) => content,
            Err(e) =>
            {
                if e.kind() == std::io::ErrorKind::NotFound
                {
                    String::new()
                }
                else
                {
                    self.app.get_log_mut()
                        .error( "Failed to read memory file" )
                        .prm( "path", &path )
                        .prm( "error", &e.to_string() );
                    String::new()
                }
            }
        }
    }



    /*
        Send memory to stdout
    */
    fn out_memory( &mut self )
    -> &mut Self
    {
        let content = self.read_memory_content();
        if content.is_empty()
        {
            println!( "No memory" );
        }
        else
        {
            println!( "{}", content );
        }
        self
    }
}
