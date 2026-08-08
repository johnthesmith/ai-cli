/*
    SPDX-License-Identifier: MIT
    SPDX-FileCopyrightText: 2026 Still Swamp
*/

/*
    Prompt secion
    It contains methods prompt
*/

impl Ai
{
    /*
        Return file name with id of prompt
    */
    fn get_prompt_file
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
                &[ "prompt-file-id" ],
                "%chat-path%/prompt.txt"
            )
            .replace( "%chat-path%", &self.get_chat_path( chat_id ))
            .replace( "%profile%", &self.get_profile())
            .replace( "%chat%", &self.get_chat() )
            .replace( "%provider%", &self.get_provider() )
            .replace( "%model%", &self.get_model_safe())
        )
    }



    /*
        Return list of provider names
    */
    fn get_prompts_path
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
                &[ "prompts-path" ],
                "%chat-path%/prompts/"
            )
            .replace( "%chat-path%", &self.get_chat_path( chat_id ) )
            .replace( "%profile-path%", &self.get_profile_path() )
            .replace( "%profile%", &self.get_profile() )
            .replace( "%chat%", chat_id )
            .replace( "%provider%", &self.get_provider() )
            .replace( "%model%", &self.get_model_safe() )
        )
    }



    /*
        Return prompt file size
    */
    fn get_prompt_content_file_size( &self )
    -> u64
    {
        let path = self.get_prompt_content_file();
        std::fs::metadata( &path )
        .map( |m| m.len() )
        .unwrap_or( 0 )
    }




    /*
        Return list of prompts
    */
    fn get_prompt_templates( &self )
    -> Vec<String>
    {
        let mut result = Vec::new();

        /* From config */
        if let Some(prompts) = self.app.config[ "prompts" ].as_object()
        {
            for key in prompts.keys()
            {
                let name = key.to_string();
                if !result.contains( &name )
                {
                    result.push(name);
                }
            }
        }

        return result;
    }



    /*
        Return list of prompts
    */
    fn get_prompts
    (
        &self,
        chat: &str
    )
    -> Vec<String>
    {
        let mut result = self.get_prompt_templates();

        /* From flies */
        let path = self.get_prompts_path( chat );
        if let Ok(entries) = std::fs::read_dir( &path )
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
                                let prompt_name = name
                                .trim_end_matches(".txt").to_string();
                                if !result.contains(&prompt_name)
                                {
                                    result.push(prompt_name);
                                }
                            }
                        }
                    }
                }
            }
        }

        if result.is_empty()
        {
            result.push("default".to_string());
        }

        result
    }


    /*
        Return current prompt id
    */
    fn read_prompt( &self )
    -> String
    {
        let prompt_path = self.get_prompt_file( &self.get_chat() );
        match std::fs::read_to_string( &prompt_path )
        {
            Ok( content ) => content,
            Err(_) => "default".to_string()
        }
    }



    fn read_prompt_content( &mut self )
    -> String
    {
        let path = self.get_prompt_content_file();
        match std::fs::read_to_string( &path )
        {
            Ok(content) => content,
            Err(e) =>
            {
                self.app.get_log_mut()
                    .error("Failed to read prompt file")
                    .prm("path", &path)
                    .prm("error", &e.to_string());
                String::new()
            }
        }
    }




    fn out_prompt
    (
        &mut self
    )
    -> &mut Self
    {
        let content = self.read_prompt_content();
        if content.is_empty()
        {
            println!("No prompt");
        }
        else
        {
            println!("{}", content);
        }
        self
    }



    /*
        Build prompt file in to current self.prompt
    */
    fn build_prompt
    (
        &mut self,
        /* Prompt name */
        template: &str
    ) -> &mut Self
    {
        let config = &self.app.config;
        let facts = &config[ "prompts" ][ template ][ "facts" ];

        if facts.is_null()
        {
            self.app.state.set_state
            (
                "prompt-template-not-found",
                json!
                (
                    {
                        "message": "Template not found for your prompt.",
                        "current-prompt-id": self.get_prompt_id(),
                        "requested-template-id": template,
                        "tip": "Your need to run `ai --build-promt=<template>`",
                        "aviable-prompt-templates": self.get_prompt_templates()
                    }
                )
            );
        }
        else
        {
            let mut dest = Storage::new( self.app.get_log_rc() );

            if let Some( facts ) = facts.as_array()
            {
                for id in facts
                {
                    let id = id.get_str( "" );
                     if let Some(( domain, actor, content ))
                     = self.fact_db.get_by_id( &id )
                     {
                         dest.insert( &id, domain, actor, &content, true );
                     }
                }
                let prompt_path = self.get_prompt_content_file();
                let _ = std::fs::write( &prompt_path, dest.to_string() );
            }
        }
        self
    }



    /*
        Write profile in to file
    */
    fn bind_prompt
    (
        &mut self,
        name: &str
    ) -> &mut Self
    {
        let path = self.get_prompt_file( &self.get_chat());

        if let Err(e) = std::fs::write( &path, name )
        {
            /* Set state for app */
            self.app.state.set_state
            (
                "prompt-write-error",
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
            .error( "Failed to write prompt id" )
            .prm( "path", &path )
            .prm( "error", &e.to_string() );
        }
        else
        {
            self.app.get_log_mut()
            .trace( "Prompt id saved" )
            .prm( "name", name );
        }

        self
    }



    /*
        Return chat for current session
    */
    fn get_prompt_id( &self )
    -> String
    {
        self.prompt_id.clone()
    }



    /*
        Set chat for current session
    */
    fn set_prompt_id
    (
        &mut self,
        id: &str
    )
    -> &mut Self
    {
        self.prompt_id = id.to_string();
        self
    }



    /*
        Return prompt file path for chat
    */
    fn get_prompt_content_file( &self )
    /* Return prompt file name */
    -> String
    {
        core::expand_path
        (
            &self.get_config_str
            (
                &[ "prompt-file" ],
                "%profile-path%/chats/%chat%/prompts/%prompt%.txt"
            )
            .replace( "%profile-path%", &self.get_profile_path() )
            .replace( "%profile%", &self.get_profile() )
            .replace( "%provider%", &self.get_provider() )
            .replace( "%model%", &self.get_model() )
            .replace( "%chat%", &self.get_chat() )
            .replace( "%prompt%", &self.get_prompt_id() )
        )
    }



    /*
       Return user prompt combining stdin pipe, CLI arguments,
       and interactive input.
    */
    fn get_user_prompt( &mut self )
    ->
    (
        /* cli input or interactive input */
        String,
        /* stdin */
        String
    )
    {
        let mut input = String::new();
        let mut stdin_data = String::new();
        let mut stdin = std::io::stdin();
        let is_pipe = !stdin.is_terminal();


        if is_pipe
        {
            match stdin.read_to_string(&mut stdin_data)
            {
                Ok(0) => {}

                Ok(_) => { stdin_data = stdin_data.trim().to_string(); }

                Err(e) =>
                {
                    self.app.get_log_mut()
                    .error("Failed to read from stdin pipe")
                    .prm("error", &e.to_string());
                }
            }
        }

        let known_keys = self.find_keys( "", "" );
        let args: Vec<String> = std::env::args().skip(1)
        .filter
        (
            |arg|
            {
                let key = arg.split('=').next().unwrap_or(arg);
                !known_keys.contains(&key.to_string())
            }
        )
        .collect();

        if !args.is_empty()
        {
            input = args.join(" ");
        }

        if !is_pipe && input.is_empty() && stdin_data.is_empty()
        {
            println!
            (
                "Enter your prompt (Ctrl+D to finish or Ctrl+C to cancel):"
            );
            let mut interactive = String::new();
            if stdin.read_to_string(&mut interactive).unwrap_or(0) > 0
            {
                input = interactive.trim().to_string();
            }
            println!();
        }

        ( input, stdin_data )
    }



    /*
        Convert a compact access string ( e.g. "siud" ) into explicit
        words joined by "|" ( e.g. "select|insert|update|delete" ),
        since a bare letter code is not reliably understood by the LLM.
        An empty access string yields "none".
    */
    fn access_to_str
    (
        &self,
        /* Compact access string, e.g. "siud" */
        access: &str
    )
    -> String
    {
        let words: Vec<&str> = access
        .chars()
        .filter_map( |c|
            match c
            {
                's' => Some( "select" ),
                'i' => Some( "insert" ),
                'u' => Some( "update" ),
                'd' => Some( "delete" ),
                _ => None,
            }
        )
        .collect();

        if words.is_empty()
        {
            "none".to_string()
        }
        else
        {
            words.join( "|" )
        }
    }




    /*
        Ensure prompt file
        - Check if prompt file exists
        - If not, create default prompt based on prompt
    */
    fn ensure_prompt_file( &mut self )
    -> &mut Self
    {
        let prompt_path = self.get_prompt_content_file();

        if
            std::fs::metadata( &prompt_path )
            .map(|m| m.len() == 0)
            .unwrap_or( true )
        {
            let prompt = self.get_prompt_id();
            self.build_prompt( &prompt );
        }

        self
    }
}
