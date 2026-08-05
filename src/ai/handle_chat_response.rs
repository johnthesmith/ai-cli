/*
    SPDX-License-Identifier: MIT
    SPDX-FileCopyrightText: 2026 Still Swamp
*/



impl Ai
{
    /*
        Processing chat response
    */
    pub fn handle_chat_response
    (
        &mut self,
        /* Content form llm */
        content: &str,
        /* Count of income tokens */
        tokens_in: u64,
        /* Count of outcome tokens */
        tokens_out: u64,
        tokens_billed: u64,
        tokens_cached: u64
    )
    {
        self.app.get_log_mut().dump( "LLM response", content );

        self.storage
        .set_access( "access", &self.access_access )
        .set_access( "history", &self.access_history )
        .set_access( "prompt", &self.access_prompt )
        .set_access( "memory", &self.access_memory )
        .set_access( "shell", &self.access_shell )
        .set_access( "clipboard", &self.access_clipboard )
        .set_access( "read", &self.access_read )
        .set_access( "write", &self.access_write )
        ;

        /* Dump table */
        self.app.get_log_mut()
        .info( "Write file mapping" )
        .prm( "mapping", &format!( "{:?}", self.write_translation ));

        /* Answer is parsing */
        let mut response_storage = Storage::new( self.app.get_log_rc() );

        response_storage
        .set_fact_delimiter( &self.fact_delimiter )
        .parse( content );

        if !response_storage.facts.is_empty()
        {
            for( id, ( domain, actor, body )) in response_storage.facts.iter()
            {
                match domain.as_str()
                {
                    "read" =>
                    {
                        self.mnemo( "^!r" );
                    }

                    "write" =>
                    {
                        if Ai::check_access( &self.access_write, "u" )
                        {
                            let file_path = self
                            .write_translation
                            .get( id.as_str() )
                            .cloned();

                            if let Some( file_path ) = file_path
                            {
                                if let Err( e )
                                = core::ensure_directory(&file_path)
                                {
                                    self.mnemo("^!w");
                                    self.app.get_log_mut()
                                    .error("Failed to create directory")
                                    .prm("path", &file_path)
                                    .prm("error", &e);
                                }
                                else
                                {
                                    match std::fs::write( &file_path, &body )
                                    {
                                        Ok(_) =>
                                        {
                                            self.mnemo( "^w" );
                                            self.app.get_log_mut()
                                            .info( "Write file updated" )
                                            .prm( "path", file_path )
                                            .prm( "id", &id )
                                            .prm( "size", body.len() )
                                            ;
                                        }
                                        Err(e) =>
                                        {
                                            self.mnemo( "^!w" );
                                            self.app.get_log_mut()
                                            .error( "Failed to write file" )
                                            .prm( "path", file_path )
                                            .prm( "id", &id )
                                            .prm( "error", &e.to_string() );
                                        }
                                    }
                                }
                            }
                            else
                            {
                                self.mnemo( "^!w" );
                                self.app.get_log_mut()
                                .error( "Write file not found in translation" )
                                .prm( "id", &id );
                            }
                        }
                        else
                        {
                            self.mnemo( "^!w" );
                            self.app.get_log_mut()
                                .warning( "Write not allowed" )
                                .prm( "id", &id );
                        }
                    }

                    "clipboard" =>
                    {
                        if Ai::check_access( &self.access_clipboard, "i" )
                        {
                            self.run_destination( &body, "clipboard", true );
                            self.mnemo( "+c" );
                        }
                        else
                        {
                            self.run_destination
                            (
                               &Color::colorize
                               (
                                    Color::Yellow,
                                    &body,
                                    Color::Default,
                                    self.colorize
                                ),
                                "message",
                                true
                            );
                            self.mnemo( "+!c" );
                        }
                    }

                    /* Execute comma1nd via destination */
                    "shell" =>
                    {
                        if
                            self.app.config[ "no-shell" ].get_bool( false ) ||
                            self.app.config[ "nsh" ].get_bool( false )
                        {
                            self.run_destination
                            (
                               &Color::colorize
                               (
                                    Color::Yellow,
                                    &body,
                                    Color::Default,
                                    self.colorize
                                ),
                                "message",
                                true
                            );
                        }
                        else
                        {
                            if !self.no_history
                            {
                                self.storage.insert
                                (
                                    "",
                                    "history",
                                    ASSISTANT,
                                    body,
                                    false
                                );
                            }

                            /* Check if command execution is disabled */
                            if self.app.config[ "no-shell" ].get_bool( false )
                            {
                                self.app.get_log_mut()
                                .info
                                (
                                    "Command execution disabled by --no-shell"
                                )
                                .prm( "exec", &body );
                            }
                            else
                            {
                                /*
                                    REMOVE_ENTER

                                    Removes newline and carriage return
                                    characters from LLM-generated command.
                                    Prevents command injection via line breaks
                                    that could:
                                    1. Terminate the current command
                                    2. Inject arbitrary new commands
                                    3. Execute hidden malicious code

                                    The cleaned command remains as a single
                                    line. Only newline/carriage return are
                                    removed all other characters (&&, |, ;, $,
                                    `, etc.) are preserved as legitimate
                                    command syntax.
                                */
                                let clean_command = body
                                .replace( '\n', " " )
                                .replace( '\r', "" )
                                ;

                                self.run_destination
                                (
                                    &clean_command,
                                    "command",
                                    false
                                );

                                if body.contains( '\n' )
                                {
                                    self.mnemo( "+!s" );
                                }
                                else
                                {
                                    self.mnemo( "+s" );
                                };
                            }
                        }
                    }

                    /* Handle memory operations */
                    "memory" =>
                    {
                        self.storage.get_state_mut().set_ok();

                        if self.no_memory
                        {
                            println!( "{}", &body );
                        }
                        else
                        {
                            let exists = self.storage.exists( &id );
                            if body.is_empty()
                            {
                                if exists
                                {
                                    self.storage.delete( &id, false );
                                    if self.storage.get_state().is_ok()
                                    {
                                        self.mnemo( "-m" );
                                        self.app.get_log_mut()
                                        .info( "Memory fact delited" )
                                        .prm( "text", &body );
                                    }
                                    else
                                    {
                                        self.mnemo( "-!m" );
                                        self.app.get_log_mut()
                                        .warning( "Memory fact wasn't delete" )
                                        .prm( "text", &body )
                                        .dump_state( self.storage.get_state() )
                                        ;
                                    }
                                }
                            }
                            else
                            {
                                if exists
                                {
                                    self.storage.update
                                    (
                                        &id,
                                        &domain,
                                        &actor,
                                        &body,
                                        false
                                    );
                                    if self.storage.get_state().is_ok()
                                    {
                                        self.mnemo( "^m" );
                                        self.app.get_log_mut()
                                        .info( "Memory fact updated" )
                                        .prm( "text", &body );
                                    }
                                    else
                                    {
                                        self.mnemo( "^!m" );
                                        self.app.get_log_mut()
                                        .warning( "Memory fact wasn't updated" )
                                        .prm( "text", &body )
                                        .dump_state( self.storage.get_state() )
                                        ;
                                    }
                                }
                                else
                                {
                                    self.storage.insert
                                    (
                                        "",
                                        &domain,
                                        &actor,
                                        &body,
                                        false
                                    );
                                    if self.storage.get_state().is_ok()
                                    {
                                        self.mnemo( "+m" );
                                        self.app.get_log_mut()
                                        .info( "Memory fact added" )
                                        .prm( "text", &body );
                                    }
                                    else
                                    {
                                        self.mnemo( "+!m" );
                                        self.app.get_log_mut()
                                        .warning( "Memory fact wasn't added" )
                                        .prm( "text", &body )
                                        .dump_state( self.storage.get_state() )
                                        ;
                                    }
                                }
                            }
                        }
                    }

                    /* Handle history operations */
                    "history" =>
                    {
                        self.storage.get_state_mut().set_ok();
                        if self.no_history
                        {
                            println!( "{}", &body );
                        }
                        else
                        {
                            let exists = self.storage.exists( &id );
                            if body.is_empty()
                            {
                                if exists
                                {
                                    self.storage.delete( &id, false );
                                    if self.storage.get_state().is_ok()
                                    {
                                        self.mnemo( "-h" );
                                        self.app.get_log_mut()
                                        .info( "History fact delited" )
                                        .prm( "text", &body );
                                    }
                                    else
                                    {
                                        self.mnemo( "-!h" );
                                        self.app.get_log_mut()
                                        .warning( "History fact wasn't delite" )
                                        .prm( "text", &body )
                                        .dump_state( self.storage.get_state() )
                                        ;
                                    }
                                }
                            }
                            else
                            {
                                self.run_destination
                                (
                                   &Color::colorize
                                   (
                                        Color::Cyan,
                                        &body,
                                        Color::Default,
                                        self.colorize
                                    ),
                                    "message",
                                    true
                                );

                                if exists
                                {
                                    self.storage.update
                                    (
                                        &id,
                                        &domain,
                                        &actor,
                                        &body,
                                        false
                                    );

                                    if self.storage.get_state().is_ok()
                                    {
                                        self.mnemo( "^h" );
                                        self.app.get_log_mut()
                                        .info( "History fact updated" )
                                        .prm( "text", &body );
                                    }
                                    else
                                    {
                                        self.mnemo( "^!h" );
                                        self.app.get_log_mut()
                                        .warning
                                        (
                                            "History fact wasn't updated"
                                        )
                                        .prm( "text", &body )
                                        .dump_state
                                        (
                                            self.storage.get_state()
                                        );
                                    }
                                }
                                else
                                {
                                    self.storage.insert
                                    (
                                        "",
                                        &domain,
                                        &actor,
                                        &body,
                                        false
                                    );

                                    if self.storage.get_state().is_ok()
                                    {
                                        self.mnemo( "+h" );
                                        self.app.get_log_mut()
                                        .info( "History fact added" )
                                        .prm( "text", &body );
                                    }
                                    else
                                    {
                                        self.mnemo( "+!h" );
                                        self.app.get_log_mut()
                                        .warning( "History fact wasn't added" )
                                        .prm( "text", &body )
                                        .dump_state
                                        (
                                            self.storage.get_state()
                                        );
                                    }
                                }
                            }
                        }
                    }

                    /* Handle prompt operations */
                    "prompt" =>
                    {
                        self.storage.get_state_mut().set_ok();

                        if self.no_prompt
                        {
                            println!( "{}", &body );
                        }
                        else
                        {
                            let exists = self.storage.exists( &id );

                            if body.is_empty()
                            {
                                if exists
                                {
                                    self.storage.delete( &id, false );
                                    if self.storage.get_state().is_ok()
                                    {
                                        self.mnemo( "-p" );
                                        self.app.get_log_mut()
                                        .info( "Prompt fact delited" )
                                        .prm( "text", &body );
                                    }
                                    else
                                    {
                                        self.mnemo( "-!p" );
                                        self.app.get_log_mut()
                                        .warning( "Prompt fact wasn't delite" )
                                        .prm( "text", &body )
                                        .dump_state
                                        (
                                            self.storage.get_state()
                                        );
                                    }
                                }
                            }
                            else
                            {
                                self.run_destination( &body, "message", true );

                                if exists
                                {
                                    self.storage.update
                                    (
                                        &id,
                                        &domain,
                                        &actor,
                                        &body,
                                        false
                                    );

                                    if self.storage.get_state().is_ok()
                                    {
                                        self.mnemo( "^p" );
                                        self.app.get_log_mut()
                                        .info( "Prompt fact updated" )
                                        .prm( "text", &body );
                                    }
                                    else
                                    {
                                        self.mnemo( "^!p" );
                                        self.app.get_log_mut()
                                        .warning( "Prompt fact wasn't updated" )
                                        .prm( "text", &body )
                                        .dump_state
                                        (
                                            self.storage.get_state()
                                        );
                                    }
                                }
                                else
                                {
                                    self.storage.insert
                                    (
                                        "",
                                        &domain,
                                        &actor,
                                        &body,
                                        false
                                    );

                                    if self.storage.get_state().is_ok()
                                    {
                                        self.mnemo( "+p" );
                                        self.app.get_log_mut()
                                        .info( "Prompt fact added" )
                                        .prm( "text", &body );
                                    }
                                    else
                                    {
                                        self.mnemo( "+!p" );
                                        self.app.get_log_mut()
                                        .warning( "Prompt fact wasn't added" )
                                        .prm( "text", &body )
                                        .dump_state
                                        (
                                            self.storage.get_state()
                                        );
                                    }
                                }
                            }
                        }
                    }

                    /* */
                    _ =>
                    {
                        self.app.get_log_mut()
                        .warning( "Formt error" )
                        .prm( "domain", domain )
                        .prm( "actor", actor )
                        .prm( "id", id )
                        .prm( "body", &body );
                        self.mnemo( "!!" );

                        let body = format!
                        (
                             "Unrecognized fact:\n{}\n{}\n{}\n{}\n",
                             domain,
                             actor,
                             id,
                             body
                        );

                        if !self.no_history
                        {
                            self.storage.insert
                            (
                                "",
                                "history",
                                ASSISTANT,
                                &body,
                                false
                            );
                        }

                        self.run_destination
                        (
                            &body,
                            "message",
                            true
                        );
                    }
                }
            }
        }
        else
        {
            self.run_destination( &content, "message", true );
            self.mnemo( "!E" );
        }

        if self.get_config_bool( &[ "out-status" ], false )
        {
            let full_status = self.status.join
            (
                &Color::colorize
                (
                    Color::Gray,
                    "|",
                    Color::Default,
                    self.colorize
                )
            );
            println!
            (
                "{}  byte h:{} | m:{} | p:{}   tokens in:{} | out:{} | bill:{} | cache:{}",
                full_status,
                self.get_history_file_size(),
                self.get_memory_file_size(),
                self.get_prompt_content_file_size(),
                tokens_in.to_string(),
                tokens_out.to_string(),
                tokens_billed.to_string(),
                tokens_cached.to_string()
            );
        }
    }
}
