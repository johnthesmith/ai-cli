/*
    SPDX-License-Identifier: MIT
    SPDX-FileCopyrightText: 2026 Still Swamp
*/

include!( "facts.rs" );

impl Ai
{
    /*
        Main run method
    */
    pub fn run( &mut self )
    -> &mut Self
    {
        self.fact_db.parse( FACTS );

        let mut actions = Vec::new();

        /* Read cli arguments */
        self.app.read_cli();

        /* No prompt mode */
        let mut no_prompt = self.app.config[ "no-prompt" ].get_bool( false );

        /* No request mode */
        let mut no_request = self.app.config[ "no-request" ].get_bool( false );

        /* Standalone mode */
        let mut is_standalone = false;

        /* Completion */
        let completion = self.app.config[ "comp-line" ].get_str( "" );
        let is_completion = !completion.is_empty();

        if !is_completion
        {
            /* Version request */
            if                self.app.config[ "version" ].get_bool( false ) ||
                self.app.config[ "v" ].get_bool( false )
            {
                println!( "{}", self.get_version() );
                is_standalone = true;
            }

            /* Help request */
            if
                self.app.config[ "?" ].get_bool( false ) ||
                self.app.config[ "h" ].get_bool( false ) ||
                self.app.config[ "help" ].get_bool( false )
            {
                self.help();
                is_standalone = true;
            }

            if self.app.config[ "tiocsti" ].get_bool( false )
            {
                /* Read from stdin */
                let mut input = String::new();
                match std::io::stdin().read_to_string( &mut input )
                {
                    Ok( 0 ) =>
                    {
                        self.app.state.set_state
                        (
                            "tiocsti-stdin-isempty",
                            json!({})
                        );
                    }
                    Ok( _ ) =>
                    {
                        self.input_tiocsti( &input );
                    }
                    Err( e ) =>
                    {
                        self.app.state.set_state
                        (
                            "tiocsti-failed-to-read-stdin",
                            json!({ "error": &e.to_string() })
                        );
                    }
                }
                is_standalone = true;
            }
        }


        /* Retrive init flag */
        let mut is_init = !self.get_profiles_path().is_empty();
        if self.app.config[ "init" ].get_bool( false )
        {
            no_prompt = true;
            no_request = true;
            /* Initialize */
            self.init();
            /* Second control */
            is_init = !self.get_profiles_path().is_empty();
        }

        if is_init && !is_standalone
        {
            if self.app.state.is_ok()
            {
                /* Let profile */
                let profile = self.app.config[ "bind-profile" ].get_str( "" );

                /* Bind profile */
                if !profile.is_empty()
                {
                    self.write_profile( &profile );
                    self.set_profile( &profile );
                    no_prompt = true;
                }

                /* Read prfile */
                if !profile.is_empty()
                {
                    /* Set profile for current session only */
                    self.set_profile( &profile );
                }
                else
                {
                    /* Read profile */
                    self.read_profile();
                }
            }



            /*
                Config section
            */
            if self.app.state.is_ok()
            {
                /* Get default config path */
                let path = self.get_config_file();

                if !std::fs::metadata(&path).is_ok()
                {
                    /* Generate config */
                    self.generate_config();
                }

                if self.app.state.is_ok()
                {
                    /* Read config */
                    self
                    .app
                    .read_config( &path )
                    .read_cli()
                    .read_sets()
                    .read_cli();
                }
            }

            /*
                Log section
            */
            if self.app.state.is_ok()
            {
                /* Set log file */
                if let Some( file ) = self.app.config
                [ "application" ][ "log" ][ "file" ].as_str()
                {
                    let file = core::expand_path( file )
                    .replace( "%profile-path%", &self.get_profile_path())
                    .replace( "%profile%", &self.get_profile())
                    ;
                    self.app.get_log_mut().set_file_path( &file );
                }
            }

            if self.app.state.is_ok()
            {
                /* First log message */
                self.app.get_log_mut().begin( "=== Ai started ===" );
                /* Dump configuration */
                self.app.dump_config();
            }



            /*
                Processing wave 1
            */
            if self.app.state.is_ok()
            {
                /* Define chat */
                self.set_chat
                (
                    &if let Some( chat )
                    = self.app.config[ "chat" ].as_str()
                    {
                        chat.to_string()
                    }
                    else
                    {
                        self.read_chat()
                    }
                );

                /* Define memory id */
                self.set_memory_id
                (
                    &if let Some( memory )
                    = self.app.config[ "memory" ].as_str()
                    {
                        memory.to_string()
                    }
                    else
                    {
                        self.read_memory_of_chat()
                    }
                );

                /* Define provider */
                self.set_provider
                (
                    &if let Some( provider )
                    = self.app.config[ "provider" ].as_str()
                    {
                        provider.to_string()
                    }
                    else
                    {
                        self.read_provider()
                    }
                );

                /* Define model */
                self.set_model
                (
                    &if let Some( model )
                    = self.app.config[ "model" ].as_str()
                    {
                        model.to_string()
                    }
                    else
                    {
                        self.read_model()
                    }
                );

                /* Define prompt */
                self.set_prompt_id
                (
                    &if let Some( prompt )
                    = self.app.config[ "prompt" ].as_str()
                    {
                        prompt.to_string()
                    }
                    else
                    {
                        self.read_prompt()
                    }
                );


                /* Compile prompt */
                let prompt_template = self.app.config[ "build-prompt" ]
                .get_str( "" );
                if !prompt_template.is_empty()
                {
                    self.build_prompt( &prompt_template );
                }
            }


            /* Processint wave 2 */
            if self.app.state.is_ok()
            {
                /* Collect actions from config into a map (copy values) */
                if let Some( mapping ) = self.app.config.as_object()
                {
                    for( key, value ) in mapping
                    {
                        let action = key.to_string();
                        let target = value.get_str( "" );
                        actions.push(( action, target ));
                    }
                }

                /* Set no prompt flag */
                for( action, _ ) in &actions
                {
                    ( no_prompt, no_request ) = match action.as_str()
                    {
                        "i" | "info" => ( true, true ),
                        "rmh" => ( true, true ),
                        "reset-history" | "rh" => ( true, true ),
                        "reset-memory" | "rm" => ( true, true ),
                        "out-history" | "oh" => ( true, true ),
                        "out-memory" | "om" => ( true, true ),
                        "out-prompt" | "op" => ( true, true ),
                        "out-prompt-content" | "opc" => ( true, true ),
                        "tiocsti" => ( true, true ),
                        "comp-line" => ( true, true ),
                        "build-prompt" => ( true, true ),
                        "bind-provider" => ( true, true ),
                        "bind-prompt" => ( true, true ),
                        "bind-model" => ( true, true ),
                        "bind-chat" => ( true, true ),
                        "bind-memory" => ( true, true ),
                        "select-fact" => ( true, true ),
                        "insert-fact" => ( true, true ),
                        "update-fact" => ( true, true ),
                        "delete-fact" => ( true, true ),
                        _ => ( no_prompt, no_request )
                    };
                }

                /* Execute actions from collected map */
                for( action, target ) in &actions
                {
                    match action.as_str()
                    {
                        "bind-chat" =>
                        {
                            self
                            .bind_chat( &target )
                            .set_chat( &target );
                        }
                        "bind-provider" =>
                        {
                            self
                            .bind_provider( &target )
                            .set_provider( &target );
                        }
                        "bind-prompt" =>
                        {
                            self
                            .bind_prompt( &target )
                            .set_prompt_id( &target );
                        }
                        "bind-model" =>
                        {
                            self
                            .bind_model( &target )
                            .set_model( &target );
                        }
                        "bind-memory" =>
                        {
                            self
                            .bind_memory( &target )
                            .set_memory_id( &target );
                        }
                        "reset-history" | "rh" =>
                        {
                            self.clear_history();
                        }
                        "reset-memory" | "rm"=>
                        {
                            self.clear_memory();
                        }
                        "rmh"=>
                        {
                            self.clear_memory().clear_history();
                        }
                        _ => {}
                    }
                }

                /* Validate provider */
                if !self.provider_exists( &self.get_provider() )
                {
                    self.app.state.set_state
                    (
                        "unknown-provider",
                        json!
                        (
                            {
                                "requested-provider": &self.get_provider()
                            }
                        )
                    );
                }
            }



            if is_completion
            {
                self.completion( &completion );
            }
            else
            {
                if self.app.state.is_ok()
                {
                    self.no_history = self.get_config_bool
                    (
                        &[ "no-history" ],
                        false
                    );

                    self.no_memory = self.get_config_bool
                    (
                        &[ "no-memory" ],
                        false
                    );

                    self.no_prompt = self.get_config_bool
                    (
                        &[ "no-prompt" ],
                        false
                    );

                    /* Set access rights */
                    self.access_access
                    = self.get_config_str( &[ "access-access" ], "s" );
                    self.access_history
                    = self.get_config_str( &[ "access-history" ], "si" );
                    self.access_memory
                    = self.get_config_str( &[ "access-memory" ], "si" );
                    self.access_prompt
                    = self.get_config_str( &[ "access-prompt" ], "s" );
                    self.access_shell
                    = self.get_config_str( &[ "access-shell" ], "i" );
                    self.access_clipboard
                    = self.get_config_str( &[ "access-clipboard" ], "i" );
                    self.access_read
                    = self.get_config_str( &[ "access-read" ], "s" );
                    self.access_write
                    = self.get_config_str( &[ "access-write" ], "su" );

                    /* Open prompt */
                    self.ensure_prompt_file();

                    /*
                        Retrive user prompt
                    */
                    let( user_input, user_stdin ) = if no_prompt
                    {
                        (String::new(), String::new())
                    }
                    else
                    {
                        self.get_user_prompt()
                    };

                    /* Build prompt */
                    self.compile_prompt( &user_input, &user_stdin );
                }


                /* User fact operation */
                if self.app.state.is_ok()
                {
                    /* Delete fact */
                    let id = self.app.config[ "delete-fact" ].get_str( "" );
                    if !id.is_empty()
                    {
                        self.storage.delete( &id, true );
                    }

                    /* Insert fact */
                    let id = self.app.config[ "insert-fact" ].get_str( "" );
                    if !id.is_empty()
                    {
                        let actor = self.app.config[ "actor" ].get_str( USER );
                        let domain = self.app.config[ "domain" ]
                        .get_str( "history" );

                        let ( std, cli ) = self.get_user_prompt();
                        let body = format!( "{} {}", std, cli);
                        let body = body.trim();
                        if !body.is_empty()
                        {
                            self.storage.insert
                            (
                                &id,
                                &domain,
                                &actor,
                                &body,
                                true
                            );
                         }
                    }

                    /* Update fact */
                    let id = self.app.config[ "update-fact" ].get_str( "" );
                    if !id.is_empty()
                    {
                        let actor = self.app.config[ "actor" ].get_str( USER );
                        let domain = self.app.config[ "domain" ]
                        .get_str( "history" );

                        let ( std, cli ) = self.get_user_prompt();
                        let body = format!( "{} {}", std, cli);
                        let body = body.trim();
                        if !body.is_empty()
                        {
                            self.storage.update
                            (
                                &id,
                                &domain,
                                &actor,
                                &body,
                                true
                            );
                        }
                    }
                }

                if self.app.state.is_ok()
                {
                    let prompt = self.storage.to_string()
                    .replace
                    (
                        "%shell%",
                        &self.get_config_str( &[ "shell" ], "/bin/bash" )
                    )

                    .replace( "%chat%", &self.get_chat() )
                    .replace( "%provider%", &self.get_provider() )
                    .replace( "%model-name%", &self.get_model_name() )
                    .replace( "%version%", &self.get_version() )
                    .replace
                    (
                        "%now%",
                        &Moment::create().now().format( "%Y-%m-%d %H:%M:%S" )
                    )
                    .replace
                    (
                        "%access-access%",
                        &self.access_to_str( &self.access_access )
                    )
                    .replace
                    (
                        "%access-history%",
                        &self.access_to_str( &self.access_history )
                    )
                    .replace
                    (
                        "%access-memory%",
                        &self.access_to_str( &self.access_memory )
                    )
                    .replace
                    (
                        "%access-prompt%",
                        &self.access_to_str( &self.access_prompt )
                    )
                    .replace
                    (
                        "%access-read%",
                        &self.access_to_str( &self.access_read )
                    )
                    .replace
                    (
                        "%access-write%",
                        &self.access_to_str( &self.access_write )
                    )
                    .replace
                    (
                        "%access-clipboard%",
                        &self.access_to_str( &self.access_clipboard )
                    )
                    .replace
                    (
                        "%access-shell%",
                        &self.access_to_str( &self.access_shell )
                    );

                    /* Execute actions from collected map */
                    for( action, target ) in &actions
                    {
                        match action.as_str()
                        {
                            "out-history" | "oh" =>
                            {
                                self.out_history();
                            }
                            "out-memory" | "om" =>
                            {
                                self.out_memory();
                            }
                            "out-prompt" | "op" =>
                            {
                                println!( "{}", prompt );
                            }
                            "out-prompt-content" | "opc" =>
                            {
                                self.out_prompt();
                            }
                            "select-fact" =>
                            {
                                if !target.is_empty()
                                {
                                    println!
                                    (
                                        "{}",
                                        self.storage.to_string_by_id( &target )
                                    );
                                }
                             }
                            _ => {}
                        }
                    }

                    /*
                        Request API section
                    */
                    if !no_request
                    {
                        let size = prompt.len();
                        let max_bytes = self.get_max_prompt_bytes();

                        if size < max_bytes
                        {
                            let mut provider = providers::create_provider
                            (
                                 &self.get_provider(),
                                 self
                            );
                            provider.chat( &prompt );
                        }
                        else
                        {
                            self.app.state.set_state
                            (
                                "prompt-size-exceededs",
                                json!
                                ({
                                    "message": "Prompt exceeds limit bytes",
                                    "size-bytes": size,
                                    "max-size-bytes": max_bytes
                                })
                            );
                        }
                    }


                    /*
                        Split and save facts
                    */
                    self.storage.split( &[ "history" ] ).save
                    (
                        &self.get_history_file_path()
                    );

                    self.storage.split( &[ "memory" ] ).save
                    (
                        &self.get_memory_file()
                    );

                    self.storage.split
                    (
                        &
                        [
                            "prompt",
                            "access",
                            "env"
                        ]
                    )
                    .save( &self.get_prompt_content_file());


                    /* Last log out */
                    self.app.get_log_mut().end( "End of ai" ).eol();
                }
            }
        }
        else
        {
            if is_completion
            {
                self.completion( &completion );
            }
            else
            {
                if !is_standalone
                {
                    self.app.state.set_state
                    (
                        "profile-not-found",
                        json!
                        (
                            {
                                "message": "Profile not found.",
                                "tip": "Your need to run `ai --init`",
                            }
                        )
                    );
                }
            }
        }

        /*
            Information about session
        */
        if
            self.app.config[ "i" ].get_bool( false ) ||
            self.app.config[ "info" ].get_bool( false )
        {
            self.out_info();
        }
        else
        {
            if !self.app.state.is_ok()
            {
                self.app.state.dump( "txt" );
            }
        }

        self
    }
}
