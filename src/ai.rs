/*
    SPDX-License-Identifier: MIT
    SPDX-FileCopyrightText: 2026 Still Swamp
*/



/*
    Main AI module
*/

mod providers;
mod config;
mod help;
mod prompts;
mod storage;

use serde_json::json;
use serde_yaml::Value;
use std::io::{ Read, Write, IsTerminal };
use core::{ App, SerdeExt, State, Moment };
use storage::Storage;

pub const USER: &str = "user";
pub const ASSISTANT: &str = "assistant";
pub const TOOL: &str = "aicli";



/*
    Ai applicatoin
*/
pub struct Ai
{
    /* Application structure */
    app: App,

    /* Profile for current session */
    profile: String,

    /* AI provider for current session */
    provider: String,

    /* Chat id */
    chat: String,

    /* Id of model of provider for current session */
    model: String,

    /* Id of current prompt */
    prompt: String,

    /* History storage */
    history_storage: Storage,

    /* Memory storage */
    memory_storage: Storage,

    /* Prompt storage */
    prompt_storage: Storage,
}



/*
    Ai implementation
*/
impl Ai
{
    /*
        ping and return AI
    */
    pub fn create() -> Self
    {
        Self
        {
            app: App::create(),
            profile: "default".to_string(),

            provider: String::new(),
            model: String::new(),
            chat: String::new(),
            prompt: String::new(),

            history_storage: Storage::new(),
            memory_storage: Storage::new(),
            prompt_storage: Storage::new()
        }
    }



    /*
        Return application
    */
    pub fn get_app( &self )
    -> &App
    {
        &self.app
    }



    /*
        Return application
    */
    pub fn get_app_mut( &mut self )
    -> &mut App
    {
        &mut self.app
    }



    /*
        Help utility
    */
    fn help( &mut self )
    -> &mut Self
    {
        println!
        (
            "{}",
            help::CONTENT.replace( "%version%", &self.get_version() )
        );
        self
    }



    /*
        Build yaml with session information and return it in to stdout
    */
    fn out_info( &mut self )
    -> &mut Self
    {
        let info = json!
        (
            {
                "version": self.get_version(),
                "net":
                {
                    "proxy":  self.read_proxy()
                },
                "session":
                {
                    "fact-delimiter": self.prompt_storage.get_fact_delimiter(),
                    "profile": self.get_profile(),
                    "provider": self.get_provider(),
                    "chat": self.get_chat(),
                    "model": self.get_model(),
                    "model-name": self.get_model_name(),
                    "prompt": self.get_prompt()
                },
                "access":
                {
                    "history": self.history_storage.get_access(),
                    "memory": self.memory_storage.get_access(),
                    "prompt": self.prompt_storage.get_access()
                },
                "files":
                {
                    "log": self.get_app().get_log().get_file_path(),
                    "config": self.get_config_file(),
                    "profile": self.get_profile_file_path(),
                    "provider": self.get_provider_file(),
                    "prompt": self.get_prompt_origin_file(),
                    "model": self.get_model_file_path(),
                    "memory": self.get_memory_file(),
                    "token": self.get_token_path(),
                    "history": self.get_history_file_path()
                },
                "statistics":
                {
                    "max_prompt_size_bytes":
                    self.get_max_chat_prompt_size_byte(),
                    "history_size_bytes":
                    self.history_storage.to_string().len(),
                    "memory_size_bytes":
                    self.memory_storage.to_string().len(),
                    "prompt_size_bytes":
                    self.prompt_storage.to_string().len()
                }
            }
        );

        println!( "{}", serde_yaml::to_string(&info).unwrap_or_default() );

        self
    }



    /*
        Main run method
    */
    pub fn run( &mut self )
    -> &mut Self
    {
        /* Read cli arguments */
        self.app.read_cli();

        /*
            Profile section
        */

        /* No prompt mode */
        let mut no_prompt = self.app
        .config[ "no-prompt" ]
        .get_bool( false );

        /* Set profile */
        let profile = self.app.config[ "switch-profile" ].get_str( "" );
        if !profile.is_empty()
        {
            self.write_profile( &profile );
            self.set_profile( &profile );
            no_prompt = true;
        }
        else
        {
            let profile = self.app.config[ "profile" ].get_str( "" );
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

        /* Get default config path */
        let path = self.get_config_file();

        /* Read config */
        self.app.read_config( &path ).read_cli();

        if !self.app.state.is_ok()
        {
            let state = self.app.state.clone();
            let error_code = state.get_code().to_string();

            self.get_app_mut().get_log_mut()
            .warning( "Configuration error" )
            .dump_state( &state )
            .eol();

            /* If config not found, try to create default config */
            if error_code == "config-not-found"
            {
                self.app.get_log_mut()
                .info( "Creating default configuration" )
                .eol();

                self.generate_config();

                /* Re-read config after creation */
                self.app.read_config( &path ).read_cli();

                if self.app.state.is_ok()
                {
                    self.get_app_mut().get_log_mut()
                    .info( "Default configuration created successfully" )
                    .eol();
                }
                else
                {
                    let state = self.app.state.clone();
                    self.app.get_log_mut()
                    .error( "Failed to create default configuration" )
                    .dump_state( &state )
                    .eol();
                }
            }
        }

        if self.app.state.is_ok()
        {
            /*
                Log section
            */
            /* Set log file */
            if let Some( file ) = self.app.config
            [ "application" ][ "log" ][ "file" ].as_str()
            {
                let file = core::expand_path( file ).replace
                (
                    "%profile%",
                    &self.get_profile()
                );
                self.app.get_log_mut().set_file_path( &file );
            }
        }


        /* First log message */
        self.app.get_log_mut().begin
        (
            "=== Ai started ==============================================="
        );


        /*
            Main section
        */
        let mut actions = Vec::new();

        /* Check config */
        if self.app.state.is_ok()
        {
            /* Dump configuration */
            self.app.dump_config();

            /* Collect actions from config into a map (copy values) */
            if let Some( mapping ) = self.app.config.as_mapping()
            {
                for( key, value ) in mapping
                {
                    let action = key.get_str( "" ).to_string();
                    let target = value.get_str( "" ).to_string();
                    actions.push(( action, target ));
                }
            }

            /* Read current values */
            self.set_chat( &self.read_chat() );
            self.set_provider( &self.read_provider() );
            self.set_model( &self.read_model() );
            self.set_prompt( &self.read_prompt() );

            let mut chat = String::new();
            let mut provider = String::new();
            let mut model = String::new();
            let mut prompt = String::new();

            /* Execute actions from collected map */
            for( action, target ) in &actions
            {
                match action.as_str()
                {
                    "p" | "provider" =>
                    {
                        provider = target.clone();
                    }

                    "m" | "model" =>
                    {
                        model = target.clone();
                    }

                    "c" | "chat" =>
                    {
                        chat = target.clone();
                    }

                    "prompt" =>
                    {
                        prompt = target.clone();
                    }

                    "switch-chat" =>
                    {
                        no_prompt = true;
                        self.switch_chat( &target );
                        chat = target.clone();
                    }

                    "switch-provider" =>
                    {
                        no_prompt = true;
                        self.switch_provider( &target );
                        provider = target.clone();
                    }

                    "switch-prompt" =>
                    {
                        no_prompt = true;
                        self.write_prompt( &target );
                        prompt = target.clone();
                    }

                    "switch-model" =>
                    {
                        no_prompt = true;
                        self.switch_model( &target );
                        model = target.clone();
                    }

                    _ => {}
                }
            }

            /* Read current values */
            if chat.is_empty()
            {
                chat = self.read_chat();
            }
            self.set_chat( &chat );

            if provider.is_empty()
            {
                provider = self.read_provider();
            }
            self.set_provider( &provider );

            if model.is_empty()
            {
                model = self.read_model();
            }
            self.set_model( &model );

            if prompt.is_empty()
            {
                prompt = self.read_prompt();
            }
            self.set_prompt( &prompt );

            /* Validate provider */
            if !self.provider_exists( &provider )
            {
                self.app.state.set_state
                (
                    "unknown-provider",
                    json!
                    (
                        {
                            "requested-provider": &provider
                        }
                    )
                );
            }
        }



        if self.app.state.is_ok()
        {
            /* Open prompt */
            self.ensure_prompt();

            let prompt_path = self.get_prompt_origin_file();
            self.prompt_storage
            .load( &prompt_path )
            .get_state()
            .state_to( &mut self.app.state );

            /* Open history */
            let history_path = self.get_history_file_path();
            self.history_storage
            .load( &history_path )
            .set_fact_delimiter( &self.prompt_storage.get_fact_delimiter() )
            .get_state()
            .state_to( &mut self.app.state );

            /* Open memory */
            let memory_path = self.get_memory_file();
            self.memory_storage
            .load( &memory_path )
            .set_fact_delimiter( &self.prompt_storage.get_fact_delimiter())
            .get_state()
            .state_to( &mut self.app.state );

            /* Execute actions from collected map */
            for( action, target ) in &actions
            {
                match action.as_str()
                {
                    "v" | "version" =>
                    {
                        no_prompt = true;
                        println!( "{}", self.get_version() );
                    }


                    "?" | "h" | "help" =>
                    {
                        no_prompt = true;
                        self.help();
                    }


                    "i" | "info" =>
                    {
                        no_prompt = true;
                        self.out_info();
                    }


                    "history" | "out-history" | "oh" =>
                    {
                        no_prompt = true;
                        self.out_history();
                    }


                    "memory" |  "out-memory" | "om" =>
                    {
                        no_prompt = true;
                        self.out_memory();
                    }


                    "prompt" | "out-prompt" | "op" =>
                    {
                        no_prompt = true;
                        let user_prompt = self.get_user_prompt();
                        println!
                        (
                            "{}",
                            self.build_prompt( &user_prompt )
                        );
                    }


                    "prompt-origin" | "out-prompt-origin" | "opo" =>
                    {
                        no_prompt = true;
                        println!
                        (
                            "{}",
                            self.prompt_storage.to_string()
                        );
                    }


                    "write-pool" =>
                    {
                        no_prompt = true;

                        let mut input = String::new();
                        if let Ok( _ )
                        = std::io::stdin().read_to_string( &mut input )
                        {
                            self.write_pool( &input );
                        }
                    }


                    "reset-history" | "rh"=>
                    {
                        no_prompt = true;
                        self.clear_history();
                    }


                    "reset-memory" | "rm"=>
                    {
                        no_prompt = true;
                        self.clear_memory();
                    }


                    "tiocsti" =>
                    {
                        no_prompt = true;
                        /* Read from stdin */
                        let mut input = String::new();
                        match std::io::stdin().read_to_string( &mut input )
                        {
                            Ok( 0 ) =>
                            {
                                self
                                .app
                                .get_log_mut()
                                .warning( "tiocsti: stdin is empty" );
                            }
                            Ok( _ ) =>
                            {
                                self.input_tiocsti( &input );
                            }
                            Err( e ) =>
                            {
                                self.app.get_log_mut()
                                .error( "tiocsti: failed to read stdin" )
                                .prm( "error", &e.to_string() );
                            }
                        }
                    }


                    "completion" =>
                    {
                        /* Generate completion mode */
                        if !target.is_empty()
                        {
                            no_prompt = true;
                            print!
                            (
                                "{}",
                                self.generate_completion( target )
                            );
                        }
                    }


                    "sh" |
                    "select-history" =>
                    {
                        no_prompt = true;
                        if !target.is_empty()
                        {
                            println!
                            (
                                "{}",
                                self.history_storage.to_string_by_id
                                (
                                    &target
                                )
                            );
                        }
                    }


                    "sm" |
                    "select-memory" =>
                    {
                        no_prompt = true;
                        if !target.is_empty()
                        {
                            println!
                            (
                                "{}",
                                self.memory_storage.to_string_by_id
                                (
                                    &target
                                )
                            );
                        }
                    }


                    "dh" |
                    "delete-history" =>
                    {
                        {
                            no_prompt = true;
                            if !target.is_empty()
                            {
                                self.history_storage.delete( &target, true );
                            }
                        }
                    }


                    "dm" |
                    "delete-memory" =>
                    {
                        {
                            no_prompt = true;
                            if !target.is_empty()
                            {
                                self.memory_storage.delete( &target, true );
                            }
                        }
                    }


                    "ih" |
                    "insert-history" =>
                    {
                        no_prompt = true;
                        let actor = self.app.config[ "actor" ]
                        .get_str( USER );

                        let body = if target.is_empty()
                        {
                            let mut input = String::new();
                            std::io::stdin().read_to_string(&mut input).ok();
                            input.trim().to_string()
                        }
                        else
                        {
                            target.clone()
                        };

                        if !body.is_empty()
                        {
                            self.history_storage.insert
                            (
                                "history",
                                "read",
                                &actor,
                                &body,
                                true
                            );
                        }
                    }


                    "im" |
                    "insert-memory" =>
                    {
                        no_prompt = true;
                        let actor = self.app.config[ "actor" ]
                        .get_str( USER );

                        let mut body = self.app.config[ "body" ]
                        .get_str( "" );

                        if body.is_empty()
                        {
                            let mut input = String::new();
                            std::io::stdin().read_to_string(&mut input).ok();
                            body = input.trim().to_string();
                        }

                        if !body.is_empty()
                        {
                            self.memory_storage.insert
                            (
                                "memory",
                                "read",
                                &actor,
                                &body,
                                true
                            );
                        }
                    }


                    "uh" |
                    "update-history" =>
                    {
                        no_prompt = true;
                        let actor = self.app.config[ "actor" ]
                        .get_str( USER );
                        let action = self.app.config[ "action" ]
                        .get_str( "read" );
                        let mut body = self.app.config[ "body" ]
                        .get_str( "" );

                        if !target.is_empty()
                        {
                            if body.is_empty()
                            {
                                let mut input = String::new();
                                std::io::stdin().read_to_string(&mut input)
                                .ok();
                                body = input.trim().to_string();
                            }
                            if !body.is_empty()
                            {
                                self.history_storage.update
                                (
                                    &target,
                                    "history",
                                    &action,
                                    &actor,
                                    &body,
                                    true
                                );
                            }
                        }
                    }


                    "um" |
                    "update-memory" =>
                    {
                        no_prompt = true;
                        let actor = self.app.config[ "actor" ]
                        .get_str( USER );

                        let action = self.app.config[ "action" ]
                        .get_str( "read" );

                        let mut body = self.app.config[ "body" ]
                        .get_str( "" );

                        if !target.is_empty()
                        {
                            if body.is_empty()
                            {
                                let mut input = String::new();
                                std::io::stdin()
                                .read_to_string( &mut input )
                                .ok();
                                body = input.trim().to_string();
                            }
                            if !body.is_empty()
                            {
                                self.memory_storage.update
                                (
                                    &target,
                                    "memory",
                                    &action,
                                    &actor,
                                    &body,
                                    true
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }

            /* Set access rights */
            let access = self.get_config_val( &[ "access" ], json!( {} ));
            self.history_storage.set_access
            (
                &access[ "history" ].get_str( "c" )
            );
            self.memory_storage.set_access
            (
                &access[ "memory" ].get_str( "c" )
            );
            self.memory_storage.set_access
            (
                &access[ "prompt" ].get_str( "" )
            );

            if !no_prompt
            {
                let provider_name = self.get_provider();
                let user_prompt = self.get_user_prompt();
                let prompt = self.build_prompt( &user_prompt );
                let max_bytes = self.get_max_chat_prompt_size_byte();
                let size = prompt.len();

                if size > max_bytes
                {
                    println!
                    (
                        "Prompt size {} bytes exceeds limit {} bytes.\n\
                         Please increase max-chat-prompt-size-byte \
                         in config,\n or run 'ai pack history \
                         --allow-history=iud' to compress conversation \
                         history.",
                        size, max_bytes
                    );
                }
                else
                {
                    /* Write user prompt to history */
                    self.history_storage.insert
                    (
                        "history",
                        "read",
                        USER,
                        &user_prompt,
                        true
                    );

                    let mut provider = providers::create_provider
                    (
                         &provider_name,
                         self
                    );
                    provider.chat( &prompt );
                }
            }

            /* Save current state */
            let history_path = self.get_history_file_path();
            self.history_storage.save( &history_path );

            /* Save current state */
            let memory_path = self.get_memory_file();
            self.memory_storage.save( &memory_path );

            /* Save current state */
//            let prompt_path = self.get_prompt_origin_file();
//            self.prompt_storage.save( &prompt_path );
        }

        /* Dump final state if its not ok*/
        if !self.app.state.is_ok()
        {
            self.app.state.dump();
        }

        /* Last log out */
        self.app.get_log_mut().end( "End of ai" ).eol();

        self
    }



    /**************************************************************************
        Prompt secion
    */

    /*
        Return file name with id of prompt
    */
    fn get_prompt_file( &self )
    -> String
    {
        core::expand_path
        (
            &self.get_config_val
            (
                &[ "prompt-file-id" ],
                "~/.local/share/ai/app/cli/%profile%/prompt.md"
                .to_string()
            )
            .replace( "%profile%", &self.get_profile() )
            .replace( "%chat%", &self.get_chat() )
            .replace( "%provider%", &self.get_provider() )
            .replace( "%model%", &self.get_model_safe() )
        )
    }



    /*
        Return currnt prompt id
    */
    fn read_prompt( &self )
    -> String
    {
        let prompt_path = self.get_prompt_file();
        match std::fs::read_to_string( &prompt_path )
        {
            Ok( content ) => content,
            Err(_) => "default".to_string()
        }
    }



    /*
        Write profile in to file
    */
    fn write_prompt
    (
        &mut self,
        name: &str
    ) -> &mut Self
    {
        let path = self.get_prompt_file();

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
    fn get_prompt( &self )
    -> String
    {
        self.prompt.clone()
    }



    /*
        Set chat for current session
    */
    fn set_prompt
    (
        &mut self,
        id: &str
    )
    -> &mut Self
    {
        self.prompt = id.to_string();
        self
    }



    /*
        Return prompt file path for chat
        Uses global prompts section with placeholders
            %profile%,
            %provider%,
            %chat%,
            %model%
            %prompt%
    */
    fn get_prompt_origin_file( &self )
    /* Return prompt file name */
    -> String
    {
        core::expand_path
        (
            &self.get_config_val
            (
                &[ "prompt-file" ],
                "~/.local/share/ai/app/cli/%profile%/prompts/%prompt%.txt"
                .to_string(),
            )
            .replace( "%profile%", &self.get_profile() )
            .replace( "%provider%", &self.get_provider() )
            .replace( "%model-name%", &self.get_model_name() )
            .replace( "%chat%", &self.get_chat() )
            .replace( "%prompt%", &self.get_prompt() )
        )
    }



    /*
       Return user prompt combining stdin pipe, CLI arguments,
       and interactive input.

        Priority:
        1. Stdin (pipe) content if available (even if empty)
        2. CLI arguments (non-flag) appended after pipe content
        3. Interactive input only when:
           - No pipe present (stdin is terminal)
           - No arguments provided
           - Result is still empty
    */
    fn get_user_prompt( &mut self )
    -> String
    {
        let mut prompt = String::new();
        let mut stdin = std::io::stdin();
        let is_pipe = !stdin.is_terminal();

        /* Read from pipe if stdin is not a terminal */
        if is_pipe
        {
            let mut pool = String::new();
            match stdin.read_to_string( &mut pool )
            {
                Ok( 0 ) =>
                {
                    /* Pipe exists but empty - do nothing, prompt stays empty */
                }
                Ok( _ ) =>
                {
                    prompt.push_str( pool.trim() );
                }
                Err( e ) =>
                {
                    self.app.get_log_mut()
                    .error( "Failed to read from stdin pipe" )
                    .prm( "error", &e.to_string());
                }
            }
        }


        /*
            Add CLI arguments (all non-flag arguments)
            Append after pipe content if both exist
        */
        let args: Vec<String> = std::env::args().skip( 1 )
            .filter( |arg| !arg.starts_with('-') )
            .collect();

        if !args.is_empty()
        {
            if !prompt.is_empty()
            {
                prompt.push(' ');
            }
            prompt.push_str( &args.join( " " ));
        }

        /*
           3. Interactive input only when:
              - No pipe (stdin is terminal)
              - No arguments were provided
              - Prompt is still empty
        */
        if !is_pipe && prompt.is_empty()
        {
            println!
            (
                "Enter your prompt (Ctrl+D to finish or Ctrl+C to cancel):"
            );

            let mut interactive = String::new();
            if stdin.read_to_string(&mut interactive).unwrap_or(0) > 0
            {
                prompt = interactive.trim().to_string();
            }
            println!();
        }

        prompt
    }



    /*
        Build full prompt from template and context
    */
    fn build_prompt
    (
        &mut self,
        /* User prompt */
        input: &str
    )
    -> String
    {
        let template = self.prompt_storage.to_request_string();

        /* Retrive shell */
        let shell = self.get_config_val
        (
            &[ "shell" ],
            "/bin/bash".to_string()
        );


        let input = input.replace
        (
            &self.prompt_storage.get_fact_delimiter(),
            "`fact-delimiter`"
        );

        let result = template
        .replace( "%history%", &self.history_storage.to_request_string() )
        .replace( "%memory%", &self.memory_storage.to_request_string() )
        .replace( "%user-prompt%", &input )
        .replace( "%shell%", &shell )
        .replace( "%chat%", &self.get_chat() )
        .replace( "%user%", USER )
        .replace( "%assistant%", ASSISTANT )
        .replace( "%tool%", TOOL )
        .replace( "%provider%", &self.get_provider() )
        .replace( "%model%", &self.get_model() )
        .replace( "%version%", &self.get_model() )
        .replace
        (
            "%max_prompt_size_byte%",
            &self.get_max_chat_prompt_size_byte().to_string()
        )
        .replace
        (
            "%now%",
            &Moment::create().now().format( "%Y-%m-%d %H:%M:%S" )
        )
        ;

        /* Calc prompt size */
        let prompt_size = result.len();

        let result = result
        .replace( "%prompt-size-byte%", &prompt_size.to_string() );

        result
    }



    /*
        Ensure prompt storage is loaded and valid
        If storage is empty:
            - Check if prompt file exists
            - If not, create default prompt based on prompt
            - Save to file and load into storage
            - If file exists but storage empty, load from file
    */
    fn ensure_prompt( &mut self )
    -> &mut Self
    {
        let prompt_path = self.get_prompt_origin_file();
        if self.prompt_storage.facts.is_empty()
        {
            if
            !std::path::Path::new( &prompt_path ).exists()
            || std::fs::metadata( &prompt_path )
            .map(|m| m.len() == 0)
            .unwrap_or(false)
            {
                let prompt = self.get_prompt();
                let default_prompt = match prompt.as_str()
                {
                    "automnemomorf" => prompts::PROMPT_AUTOMNEMOMORF.to_string(),
                    _ => prompts::PROMPT_DEFAULT.to_string(),
                };

                let _ = std::fs::write(&prompt_path, &default_prompt);
                self.prompt_storage.parse_file( &default_prompt );
                self.prompt_storage.save( &prompt_path );
            }
            else
            {
                self.prompt_storage.load(&prompt_path);
            }
        }

        self
    }




    /**************************************************************************
        History
    */

    fn get_history_file_path( &self )
    -> String
    {
        core::expand_path
        (
            &self.get_config_val
            (
                &[ "history" ],
                "~/.config/ai/app/cli/%profile%/history/%chat%.txt".to_string()
            )
            .replace( "%profile%", &self.get_profile() )
            .replace( "%model%", &self.get_model_safe() )
            .replace( "%provider%", &self.get_provider() )
            .replace( "%chat%", &self.get_chat() )
        )
    }



    /*
        Clear history file
    */
    fn clear_history( &mut self ) -> &mut Self
    {
        /* Clear all facts from memory */
        self.history_storage.clear();
        self.app.get_log_mut().info( "History cleared" );

        self
    }


    /*
        Send history to stdout
    */
    fn out_history( &mut self )
    -> &mut Self
    {
        let history = self.history_storage.to_string();
        if history.is_empty()
        {
            println!( "No history" );
        }
        else
        {
            println!( "{}", history );
        }
        self
    }




    /*******************************************************************8******
        pools
    */



    /*
        Return pool file
    */
    fn get_pool_path( &self )
    -> String
    {
        core::expand_path
        (
            &self.get_config_val
            (
                &[ "pool" ],
                "~/.local/share/ai/app/cli/%profile%/pool.txt".to_string()
            )
            .replace( "%profile%", &self.get_profile() )
            .replace( "%provider%", &self.get_provider() )
            .replace( "%model%", &self.get_model_safe() )
            .replace( "%chat%", &self.get_chat() )
        )
    }



    fn write_pool
    (
        &mut self,
        data: &str
    )
    {
        let pool_path = self.get_pool_path();
        if let Some(parent) = std::path::Path::new(&pool_path).parent()
        {
            let _ = std::fs::create_dir_all(parent);
        }

        match std::fs::write(&pool_path, data)
        {
            Ok(_) =>
            {
                self.app.get_log_mut()
                    .info( "pool written to file" )
                    .prm( "path", &pool_path );
            }
            Err(e) =>
            {
                self.app.get_log_mut()
                    .error( "Failed to write pool" )
                    .prm( "path", &pool_path )
                    .prm( "error", &e.to_string() );
            }
        }

        print!( "{}", data );
    }




    /**************************************************************************
        Any
    */


    /*
        Return config file path for current profile
    */
    fn get_version( &self )
    -> String
    {
        format!( "AI CLI Utility v{}", env!( "CARGO_PKG_VERSION" ))
    }



    /*
        Return config file path for current profile
    */
    fn get_config_file( &self )
    -> String
    {
        core::expand_path( "~/.config/ai/app/cli/%profile%/config.yaml" )
        .replace( "%profile%", &self.get_profile())
    }



    /*
        Generate config file
    */
    fn generate_config(&mut self) -> &mut Self
    {
        let path = self.get_config_file();

        if let Err(e) = core::ensure_directory(&path)
        {
            self.app.state.set_state
            (
                "config-dir-create-error",
                json!({ "error": e.to_string(), "path": path })
            );
            return self;
        }

        match std::fs::write(&path, config::DEFAULT.as_bytes())
        {
            Ok(_) =>
            {
                self.app.state = State::ok();
            }
            Err(e) =>
            {
                self.app.state.set_state(
                    "config-write-error",
                    json!({ "error": e.to_string(), "path": path })
                );
            }
        }

        self
    }


    /*
        Return proxy for current provider
    */
    fn read_proxy( &self )
    -> String
    {
        self.get_config_val( &[ "proxy" ], String::new() )
    }



    /*
        Get configuration value with inheritance:
        1. provider.model.chat.key
        2. provider.model.key
        3. provider.key
        4. global key
        5. default
    */
    fn get_config_val<T: Clone + serde::de::DeserializeOwned>
    (
        &self,
        /* Key path after "app.ai" */
        keys: &[&str],
        /* Default value */
        default: T,
    )
    -> T
    {
        let config = &self.app.config;
        let ai_cfg = &config[ "application" ][ "ai" ];

        let provider = self.get_provider();
        let model = self.get_model();
        let chat = self.get_chat();

        let get_nested = |root: &Value| -> Option<Value>
        {
            let mut current = root;
            for &k in keys
            {
                current = current.get(k)?;
            }
            Some(current.clone())
        };

        /* 1. provider.model.chat.key */
        if let Some(val) = get_nested
        (
            &ai_cfg
            [ "providers" ]
            [ &provider ]
            [ "models" ]
            [ &model ]
            [ "chats" ]
            [ &chat ]
        )
        {
            if let Ok( v ) = serde_yaml::from_value( val )
            {
                return v;
            }
        }

        /* 2. provider.model.key */
        if let Some(val) = get_nested
        (
            &ai_cfg["providers"][&provider]["models"][&model]
        )
        {
            if let Ok( v ) = serde_yaml::from_value( val )
            {
                return v;
            }
        }

        /* 3. provider.key */
        if let Some(val) = get_nested(&ai_cfg["providers"][&provider])
        {
            if let Ok( v ) = serde_yaml::from_value( val )
            {
                return v;
            }
        }

        /* 4. global key */
        if let Some( val ) = get_nested( ai_cfg )
        {
            if let Ok(v) = serde_yaml::from_value( val )
            {
                return v;
            }
        }

        default
    }




    /*******************************************************************8******
        Token
    */

    /*
        Return token path for current provider
    */
    fn get_token_path( &self ) -> String
    {
        let default = "~/.config/ai/app/cli/%profile%/tokens/%provider%.txt".to_string();
        let path = self.get_config_val( &[ "token" ], default );
        core::expand_path
        (
            &path
            .replace( "%profile%", &self.get_profile())
            .replace( "%chat%", &self.get_chat())
            .replace( "%provider%", &self.get_provider())
            .replace( "%model%", &self.get_model_safe())
        )
    }



    /*************************************************************************
        Model
    */

    /*
        Return file for current model
        Placeholders: %profile%, %provider%, %chat%
    */
    fn get_model_file_path( &self )
    -> String
    {
        let default
        = "~/.local/share/ai/app/cli/%profile%/models/%provider%.txt"
        .to_string();

        let path = self.get_config_val( &[ "model-file" ], default );

        core::expand_path
        (
            &path
            .replace( "%profile%", &self.get_profile() )
            .replace( "%provider%", &self.get_provider() )
            .replace( "%chat%", &self.get_chat() )
        )
    }



    pub fn read_model( &self ) -> String
    {
        let path = self.get_model_file_path();

        if let Ok(content) = std::fs::read_to_string( &path )
        {
            let model = content.trim().to_string();
            if !model.is_empty()
            {
                return model;
            }
        }
        "default".to_string()
    }



    /*
        Change current model
    */
    fn switch_model
    (
        &mut self,
        id: &str
    )
    -> &mut Self
    {
        let file_path = self.get_model_file_path();

        if let Err(e) = core::ensure_directory( &file_path )
        {
            self.app.get_log_mut()
            .error( "Failed to ensure model directory" )
            .prm( "error", &e );
            return self;
        }

        if let Err(e) = std::fs::write( &file_path, id )
        {
            self.app.get_log_mut()
            .error( "Failed to switch model" )
            .prm( "path", &file_path )
            .prm( "error", &e.to_string() );
        }
        else
        {
            self.app.get_log_mut()
            .trace( "Model switched" )
            .prm( "id", id);
        }

        self
    }



    /*
        Return model
    */
    fn get_model( &self )
    -> String
    {
        self.model.clone()
    }



    /*
        Return safe model (replace special chars for filesystem)
    */
    fn get_model_safe( &self )
    -> String
    {
        self.get_model()
        .replace( '/', "_" )
        .replace( '\\', "_" )
        .replace( '.', "_" )
        .replace( "..", "_" )
    }



    /*
        Set modle for current session
    */
    fn set_model
    (
        &mut self,
        name: &str
    ) -> &mut Self
    {
        self.model = name.to_string();
        self
    }



    /*
        Return return model name by current model alias
    */
    fn get_model_name( &self )
    -> String
    {
        self.app.config
        [ "application" ]
        [ "ai" ]
        [ "providers" ]
        [ self.get_provider() ]
        [ "models" ]
        [ self.get_model() ].get_str( "unknown" )
    }



    /*************************************************************************
        Provider
    */

    /*
        Return provider
    */
    fn get_provider( &self )
    -> String
    {
        self.provider.clone()
    }



    /*
        Set provider for current session
    */
    fn set_provider
    (
        &mut self,
        name: &str
    )
    -> &mut Self
    {
        self.provider = name.to_string();
        self
    }



    /*
        Return provider file path
    */
    fn get_provider_file( &self ) -> String
    {
        let path = self.get_config_val
        (
            &[ "provider-file" ],
            "~/.local/share/ai/app/cli/%profile%/provider.txt".to_string()
        );

        core::expand_path( &path.replace( "%profile%", &self.get_profile() ))
    }



    /*
        Check provider exists at config file
    */
    fn provider_exists
    (
        &self,
        /* Provider id */
        id: &str
    )
    -> bool
    {
        let config = &self.app.config;
        let provider = &config[ "application" ][ "ai" ][ "providers" ][ id ];
        !provider.is_null()
    }



    /*
        Return provider
    */
    fn read_provider( &self )
    -> String
    {
        let path = self.get_provider_file();

        if let Ok(content) = std::fs::read_to_string(&path)
        {
            let provider = content.trim().to_string();
            if !provider.is_empty()
            {
                return provider;
            }
        }
        "github".to_string()
    }



    /*
        Change current provider
    */
    fn switch_provider
    (
        &mut self,
        new_provider: &str
    ) -> &mut Self
    {
        let file_path = self.get_provider_file();

        if let Err( e ) = core::ensure_directory( &file_path )
        {
            self.app.get_log_mut()
            .error( "Failed to ensure provider directory" )
            .prm( "error", &e );
            return self;
        }

        if let Err( e ) = std::fs::write( &file_path, new_provider )
        {
            self.app.get_log_mut()
            .error( "Failed to switch provider" )
            .prm( "path", &file_path )
            .prm( "error", &e.to_string() );
        }
        else
        {
            self.app.get_log_mut()
            .trace( "Provider switched" )
            .prm( "provider", new_provider );
        }

        self
    }



    /**************************************************************************
        Profile
    */

    /*
        Return profile file
    */
    fn get_profile_file_path( &self )
    -> String
    {
        core::expand_path( "~/.local/share/ai/app/cli/profile" )
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
        let path = self.get_profile_file_path();

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
        let path = self.get_profile_file_path();

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



    /*******************************************************************8******
        Chat
    */

    /*
        Return file for current chat id
    */
    fn get_chat_file_path( &self )
    -> String
    {
        let path = self.get_config_val
        (
            &["chat-file"],
            "~/.local/share/ai/app/cli/%profile%/chat.txt".to_string()
        );

        core::expand_path
        (
            &path
            .replace( "%profile%", &self.get_profile() )
        )
    }


    /*
        Return max length for chat prompt
    */
    fn get_max_chat_prompt_size_byte( &self )
    -> usize
    {
        self.get_config_val( &[ "max-chat-prompt-size-byte" ], 100000 )
    }




    fn read_chat( &self )
    -> String
    {
        let path = self.get_chat_file_path();

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
    fn switch_chat
    (
        &mut self,
        new_id: &str
    )
    -> &mut Self
    {
        let file_path = self.get_chat_file_path();

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
            .error( "Failed to switch chat" )
            .prm( "path", &file_path)
            .prm( "error", &e.to_string());
        }
        else
        {
            self.app.get_log_mut()
            .trace( "Chat switched" )
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



    /**************************************************************************
        Commands
    */

    /*
        Run destination command by identifier.
        Identifier: "command", "out", "pool"
    */
    fn run_destination
    (
        &mut self,
        data: &str,
        dest_type: &str,
        /* true for sync execute or false for async */
        wait: bool
    )
    {
        let command = self.get_config_val
        (
            &[ "destination", dest_type ],
            String::new()
        );
        self.run_command( data, &command, wait );
    }



    /*
        Execute external command to insert the AI-generated text.
        Falls back to stdout if command execution fails.
    */
    fn run_command
    (
        &mut self,
        /* Data written to command's STDIN */
        data: &str,
        /* Command line for execution (passed to shell -c) */
        command: &str,
        /* true for sync execute or false for async */
        wait: bool
    )
    {
        if command.is_empty()
        {
            println!( "{}", data );
            return;
        }

        /* Retrive shell */
        let shell = self.get_config_val(&[ "shell" ], "/bin/bash".to_string() );

        /* Replace data in command */
        let data_arg = &data.replace( '"', "\"" );
        let run_command = &command.replace( "%data%", data_arg );

        match std::process::Command::new
        (
            shell
        )
        .arg( "-c" )
        .arg( run_command )
        .stdin( std::process::Stdio::piped() )
        .spawn()
        {
            Ok(mut child) =>
            {
                let data_len = data.len();

                if let Some( mut stdin ) = child.stdin.take()
                {
                    let _ = stdin.write_all(data.as_bytes());
                    let _ = stdin.flush();
                }

                if wait
                {
                    match child.wait()
                    {
                        Ok( exit_status ) =>
                        {
                            self.app.get_log_mut()
                            .info( "Command executed successfully" )
                            .prm( "command", command )
                            .prm( "data_bytes", data_len )
                            .prm
                            (
                                "exit_code",
                                exit_status.code().unwrap_or( -1 )
                            );
                        }
                        Err( e ) =>
                        {
                            self.app.get_log_mut()
                            .warning( "Failed to wait for command" )
                            .prm( "command", run_command)
                            .prm( "error", &e.to_string());
                        }
                    }
                }
                else
                {
                    self.app.get_log_mut()
                    .info( "Command spawned (no wait)" )
                    .prm( "command", run_command)
                    .prm( "data_bytes", data_len);

                    std::thread::spawn
                    (
                        move ||
                        {
                            let _ = child.wait();
                        }
                    );
                }
            }
            Err( e ) =>
            {
                self
                .app.get_log_mut()
                .error( "Failed to execute command" )
                .prm( "command", run_command )
                .prm( "data_bytes", data.len() )
                .prm( "error", &e.to_string() );
                println!( "{}", data );
            }
        }
    }



    /*
        Format text to maximum line width of N characters (not bytes)
        Splits at word boundaries when possible
    */
    #[allow(dead_code)]
    fn format_text(&self, text: &str, max_chars: usize) -> String
    {
        let mut result = String::new();
        let mut line = String::new();

        for word in text.split_whitespace() {
            // Check length in characters, not bytes
            let new_len = line.chars().count() + word.chars().count() + 1;

            if new_len > max_chars
            {
                if !line.is_empty()
                {
                    result.push_str(&line);
                    result.push('\n');
                    line.clear();
                }
                line.push_str(word);
            }
            else
            {
                if !line.is_empty()
                {
                    line.push(' ');
                }
                line.push_str(word);
            }
        }

        if !line.is_empty()
        {
            result.push_str(&line);
        }

        result
    }



    /*
        Processing chat response
    */
    pub fn handle_chat_response
    (
        &mut self,
        /* Content form llm */
        content: &str
    )
    {
        let content = &content.replace( "%pool%", &self.get_pool_path() );
        self.app.get_log_mut().dump( "LLM response", content );

        let mut storage = Storage::new();
        storage.parse_answer( content );

        if !storage.facts.is_empty()
        {
            let mut mnemonics: Vec<String> = Vec::new();

            for( id, ( origin, action, actor, body )) in storage.facts.iter()
            {
                match( origin.as_str(), action.as_str())
                {
                    ( "pool", "add" ) =>
                    {
                        self.run_destination( &body, "pool", true );
                        mnemonics.push( "p+".to_string() );
                    }

                    ( "clipboard", "add" ) =>
                    {
                        self.run_destination( &body, "clipboard", true );
                        mnemonics.push( "c-".to_string() );
                    }

                    /* Execute command via destination */
                    ( "shell", "add" ) =>
                    {
                        self.history_storage.insert
                        (
                            "history",
                            "read",
                            ASSISTANT,
                            body,
                            false
                        );
                        /* Check if command execution is disabled */
                        if self.app.config[ "no-shell" ].get_bool( false )
                        {
                            self.app.get_log_mut()
                            .info( "Command execution disabled by --no-shell" )
                            .prm( "exec", &body );
                        }
                        else
                        {
                            /*
                                REMOVE_ENTER

                                Removes newline and carriage return characters
                                from LLM-generated command. Prevents command
                                injection via line breaks that could:
                                1. Terminate the current command
                                2. Inject arbitrary new commands
                                3. Execute hidden malicious code

                                The cleaned command remains as a single line.
                                Only newline/carriage return are removed all
                                other characters (&&, |, ;, $, `, etc.) are
                                preserved as legitimate command syntax.
                            */
                            let clean_command = body
                            .replace( '\n', " " )
                            .replace( '\r', "" )
//                            .replace( ' ', "\\ ")
                            ;

                            self.run_destination
                            (
                                &clean_command,
                                "command",
                                false
                            );

                            if body.contains( '\n' )
                            {
                                mnemonics.push( "s!".to_string() );
                            }
                            else
                            {
                                mnemonics.push( "s+".to_string() );
                            };
                        }
                    }

                    /* Handle memory operations */
                    ( "memory", "add" ) =>
                    {
                        self.memory_storage.insert
                        (
                            "memory",
                            "read",
                            "%assistant%",
                            &body,
                            false
                        );
                        mnemonics.push( "m+".to_string() );
                        self.app.get_log_mut()
                        .info( "Memory entry added" )
                        .prm( "text", &body );
                    }


                    /* Handle history operations */
                    ( "history", "add" ) =>
                    {
                        self.history_storage.insert
                        (
                            "history",
                            "read",
                            "%assistant%",
                            &body,
                            false
                        );
                        self.run_destination( &body, "message", true );
                        mnemonics.push( "h+".to_string() );

                        self.app.get_log_mut()
                        .info( "History entry added" )
                        .prm( "text", &body );
                    }

                    /* Handle prompt operations */
                    ( "prompt", "add" ) =>
                    {
                        self.prompt_storage.insert
                        (
                            "prompt",
                            "read",
                            "%assistant%",
                            &body,
                            false
                        );
                        mnemonics.push( "p+".to_string() );
                        self.app.get_log_mut()
                        .info( "Prompt entry added" )
                        .prm( "text", &body );
                    }

                    /* Remove entries by ID */
                    ( _, "remove" ) =>
                    {
                        if self.memory_storage.exists( &id)
                        {
                            self.memory_storage.delete( &id, false );
                            mnemonics.push( "m-".to_string());
                        }

                        if self.prompt_storage.exists( &id)
                        {
                            self.prompt_storage.delete( &id, false);
                            mnemonics.push( "p-".to_string());
                        }

                        if self.history_storage.exists( &id)
                        {
                            self.history_storage.delete( &id, false );
                            mnemonics.push( "h-".to_string() );
                        }
                    }

                    /* Change entries by ID */
                    ( _, "change" ) =>
                    {
                        if self.memory_storage.exists( &id)
                        {
                            self.memory_storage.update
                            (
                                &id,
                                "memory",
                                "read",
                                &actor,
                                &body,
                                false
                            );
                            mnemonics.push( "m#".to_string() );
                            self.app.get_log_mut()
                            .info( "Memory entry changed" )
                            .prm( "id", &id );
                        }

                        if self.prompt_storage.exists( &id)
                        {
                            self.prompt_storage.update
                            (
                                &id,
                                "prompt",
                                "read",
                                &actor,
                                &body,
                                false
                            );
                            mnemonics.push( "p#".to_string() );
                            self.app.get_log_mut()
                            .info( "Prompt entry changed" )
                            .prm( "id", &id );
                        }

                        if self.history_storage.exists( &id)
                        {
                            self.history_storage.update
                            (
                                &id,
                                "history",
                                "read",
                                &actor,
                                &body,
                                false
                            );
                            mnemonics.push( "h#".to_string() );
                            self.app.get_log_mut()
                            .info( "History entry changed" )
                            .prm( "id", &id );
                        }
                    }

                    /* */
                    _ =>
                    {
                        self.app.get_log_mut()
                        .warning( "Formt error" )
                        .prm( "id", id )
                        .prm( "origin", origin )
                        .prm( "action", action )
                        .prm( "actor", actor )
                        .prm( "body", &body );

                        mnemonics.push( "?".to_string() );

                        let body = format!
                        (
                             "Unrecognized fact:\n{}\n{}\n{}\n{}\n{}\n",
                             id,
                             origin,
                             action,
                             actor,
                             body
                         );

                        self.history_storage.insert
                        (
                            "history",
                            "read",
                            ASSISTANT,
                            &body,
                            false
                        );

                        self.run_destination
                        (
                            &body,
                            "message",
                            true
                        );
                    }
                }
            }
            if self.get_config_val( &[ "show-mnemonic" ], false )
            {
                let full_mnemonic = mnemonics.join( "|" );
                println!
                (
                    "{} |  h:{} / m:{} / p:{}",
                    full_mnemonic,
                    self.history_storage.to_string().len(),
                    self.memory_storage.to_string().len(),
                    self.prompt_storage.to_string().len()
                );
            }
        }
    }



    /*
        Inject command directly into TTY input pool using TIOCSTI ioctl.

        This makes the command appear in the user's terminal prompt as if typed.
        Does NOT press Enter - user can edit before executing.

        # Security Warning
        Requires `sudo sysctl -w dev.tty.legacy_tiocsti=1` on modern kernels.
        Disabled by default due to security risks. Only use in trusted
        environments.

        # Arguments
        * `cmd` - Command string to inject (without newline)
    */
    fn input_tiocsti
    (
        &mut self,
        cmd: &str
    )
    {
        // Clone the config value to avoid borrowing self
        let tty_device = self.get_config_val
        (
            &[ "tty_device" ],
            "/dev/tty".to_string()
        );

        match std::fs::OpenOptions::new().write( true ).open( &tty_device )
        {
            Ok(fd) =>
            {
                use std::os::unix::io::AsRawFd;
                let fd_raw = fd.as_raw_fd();
                for byte in cmd.bytes()
                {
                    let ret = unsafe
                    {
                        libc::ioctl(fd_raw, libc::TIOCSTI, &byte)
                    };
                    if ret != 0
                    {
                        self.app.get_log_mut()
                        .error( "TIOCSTI ioctl failed" )
                        .prm( "byte", &byte.to_string())
                        .prm
                        (
                            "error",
                             &std::io::Error::last_os_error().to_string()
                         );

                        break;
                    }
                }

                self.app.get_log_mut()
                .info( "Command injected via TIOCSTI" )
                .prm( "tty", &tty_device )
                .prm( "length", cmd.len() );
            }
            Err(e) =>
            {
                self.app.get_log_mut()
                .error( "Failed to open TTY device" )
                .prm( "device", &tty_device )
                .prm( "error", &e.to_string() );
                println!( "{}", cmd );
            }
        }
    }



    /**************************************************************************
        Memory
    */

    /*
        Return memory file path for current chat
        Supports %profile% and %chat% placeholders
        Default: ~/.local/share/ai/app/cli/%profile%/memory/%chat%.txt
    */
    fn get_memory_file( &self )
    -> String
    {
        core::expand_path
        (
            &self.get_config_val
            (
                &[ "memory" ],
                "~/.local/share/ai/app/cli/%profile%/memory/%chat%.txt"
                .to_string()
            )
            .replace( "%profile%", &self.get_profile() )
            .replace( "%provider%", &self.get_provider() )
            .replace( "%model%", &self.get_model_safe() )
            .replace( "%chat%", &self.get_chat() )
        )
    }



    /*
        Clear memory file for current chat
    */
    fn clear_memory( &mut self )
    -> &mut Self
    {
        /* Clear all facts from memory */
        self.memory_storage.clear();
        self.app.get_log_mut().info( "Memory cleared" );
        self
    }



    /*
        Send memory to stdout
    */
    fn out_memory( &mut self )
    -> &mut Self
    {
        let memory = self.memory_storage.to_request_string();
        if memory.is_empty()
        {
            println!( "No memory" );
        }
        else
        {
            println!( "{}", memory );
        }
        self
    }



    /**************************************************************************
        Providers methods
    */

    /*
        Event triggered before sending HTTP request to LLM provider.
        Logs the prompt for debugging and audit purposes.
    */
    pub fn on_before_request
    (
        &mut self,
        /* Full prompt text that will be sent to LLM */
        prompt: &str,
        /* Provider name */
        provider: &str,
        /* Model identifier */
        model: &str,
        /* API endpoint URL for the request */
        api_url: &str
    )
    {
        self.app.get_log_mut()
        .dump( "Prompt to LLM", prompt )
        .prm( "provider", provider )
        .prm( "model", model )
        .prm( "api", api_url )
        ;
    }



    /*
        Event triggered after receiving HTTP response from LLM provider.
        Logs the response for debugging and audit purposes.
    */
    pub fn on_after_response
    (
        &mut self,
        /* Raw response text from LLM */
        response: &str,
        /* Provider name (e.g., "github", "openai", "deepseek" ) */
        provider: &str,
        /* Model identifier used for the request */
        model: &str,
        /* API endpoint URL used for the request */
        api_url: &str,
        /* Promt id */
        prompt: &str
    )
    {
        self.app.get_log_mut()
        .dump( "Response from LLM", response )
        .prm( "provider", provider )
        .prm( "model", model )
        .prm( "api", api_url )
        .prm( "promtp", prompt );
    }



    /**************************************************************************
        Completion
    */


    /*
        Generate shell completion string
    */
    fn generate_completion
    (
        &self,
        shell: &str
    ) -> String
    {
        let options =
        [
            // Help & Info
            "-?",
            "-h",
            "--help",
            "-i",
            "--info",
            "-v",
            "--version",

            // Session control
            "--no-prompt",
            "--no-command",

            // Profile & provider & model & chat (temporary)
            "--profile=",
            "--provider=",
            "--model=",
            "--chat=",

            // Permanent switch
            "--switch-profile=",
            "--switch-provider=",
            "--switch-model=",
            "--switch-chat=",

            // LLM access rights (iud)
            "--access-history=",
            "--access-memory=",
            "--access-prompt=",

            // Storage operations: history
            "-oh",
            "--out-history",
            "-ch",
            "--clear-history",
            "--select-history=",
            "--delete-history=",
            "--update-history=",
            "--insert-history=",

            // Storage operations: memory
            "-om",
            "--out-memory",
            "-cm",
            "--clear-memory",
            "--select-memory=",
            "--delete-memory=",
            "--update-memory=",
            "--insert-memory=",

            // Prompt
            "-op",
            "--out-prompt",
            "-opo",
            "--out-prompt-origin",

            // Actor & body for insert/update
            "--actor=",
            "--body=",

            // Specific features
            "--write-pool",
            "--tiocsti",

            // Shell completion
            "--completion=bash",
            "--completion=zsh",
            "--completion=fish",
        ];
        let options_str = options.join( " " );

        match shell
        {
            "bash" => self.generate_bash_completion( &options_str ),
            "zsh" => self.generate_zsh_completion( &options ),
            "fish" => self.generate_fish_completion( &options ),
            _ => format!
            (
                "Unsupported shell: {}. Supported: bash, zsh, fish\n",
                shell
            )
        }
    }



    /*
        Generate bash completion
    */
    fn generate_bash_completion
    (
        &self,
        options: &str
    )
    -> String
    {
        [
            "_ai() {",
            "    local cur prev words cword",
            "    _init_completion || return",
            &format!
            (
                 "    COMPREPLY=($(compgen -W '{}' -- \"$cur\"))",
                 options
            ),
            "}",
            "complete -F _ai ai",
            "complete -F _ai 1\n",
        ]
        .join( "\n" )
    }



    /*
        Generate zsh completion
    */
    fn generate_zsh_completion
    (
        &self,
        options: &[ &str ]
    )
    -> String
    {
        let args = options.iter()
            .map(|o| format!( "  '{}'", o ))
            .collect::<Vec<_>>()
            .join( " \\\n" );

        [
            "#compdef ai",
            "_ai() {",
            "    local line",
            "    _arguments \\",
            &args,
            "        '*: :->args'",
            "}",
            "_ai\n",
        ]
        .join( "\n" )
    }



    /*
        Generate fish completion
    */
    fn generate_fish_completion
    (
        &self, options: &[ &str ]
    ) -> String
    {
        let mut fish = String::new();
        for opt in options
        {
            let opt_clean = opt.trim_end_matches( '=' );
            fish.push_str
            (
                &format!
                (
                    "complete -c ai -f -a '{}'\n", opt_clean
                )
            );
            if opt.ends_with( '=' )
            {
                fish.push_str
                (
                    &format!
                    (
                        "complete -c ai -f -a '{}<'\n",
                         opt_clean
                     )
                );
            }
        }
        fish.push_str( "complete -c 1 -f -a '$(complete -C ai)'\n" );
        fish
    }



    /*
        Return request timeout in milliseconds
    */
    fn get_request_timeout_ms( &self ) -> u64
    {
        self.get_config_val( &[ "request_timeout_ms" ], 30000 )
    }



    /*
        Return connect timeout in milliseconds
    */
    fn get_connect_timeout_ms( &self ) -> u64
    {
        self.get_config_val( &[ "connect_timeout_ms" ], 10000 )
    }
}
