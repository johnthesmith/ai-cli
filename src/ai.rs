/*
    SPDX-License-Identifier: MIT
    SPDX-FileCopyrightText: 2026 Still Swamp
*/



/*
    Main AI module
*/

mod providers;
mod config;
mod prompts;
mod storage;

use serde_json::json;
use serde_yaml::Value;
use std::io::{ Read, Write, IsTerminal };
use core::{ App, SerdeExt, State, Moment };
use storage::Storage;


pub const BLOCK_DELIMITER: &str = "===AIOL9B1MZX===";

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

    /* History storage */
    history_storage: Storage, 

    /* Memory storage */
    memory_storage: Storage, 
}



/*
    Ai implementation
*/
impl Ai 
{
    /*
        Create and return AI
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
            history_storage: Storage::new( BLOCK_DELIMITER ),
            memory_storage: Storage::new( BLOCK_DELIMITER )
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
        println!("{}\n", self.get_version());
        println!("");
        println!("Usage:");
        println!("    ai                         Interactive keyboard input");
        println!("    ai <question>              Ask a question");
        println!("    echo <text> | ai           Read from stdin");
        println!("    ai --help                  Show this help");
        println!("Pattern:");
        println!("    ai [ message] [--<action>=<key>][--<argument>=<value>]");
        println!("");
        println!("Optins:");
        println!("    --help|-?|-h               Same as --show=help");
        println!("    --info|-h                  Same as --show=info");
        println!("    --version|-v               Same as --show=version");
        println!("");
        println!("    --no-prompt                Suppress input user prompt");
        println!("    --no-command               Suppress command event");
        println!("");
        println!("Session:");
        println!("    --profile=<id>             Use profile for current session only");
        println!("    --provider|-p=<id>         Use provider for current session only");
        println!("    --model|-m=<id>            Use model for current session only");
        println!("    --chat|-c=<id>             Use chat for current session only");
        println!("    --switch-profile=<id>      Permanently switch profile");
        println!("    --switch-provider=<id>     Permanently switch provider");
        println!("    --switch-model=<id>        Permanently switch model");
        println!("    --switch-chat=<id>         Permanently switch chat");
        println!("");
        println!("Access for LLM");
        println!("    --access-history=<mode>    Set history access rights (c=create, u=update, d=delete)");
        println!("                               Example: --access-history=cud");
        println!("    --access-memory=<mode>     Set memory access rights (c=create, u=update, d=delete)");
        println!("                               Example: --access-memory=cud");
        println!("");
        println!("Storage operations with target history|memory" );
        println!("    --history                  Show full history for chat");
        println!("    --memory                   Show full memory" );
        println!("    --clear-history            Remove history content for current chat" );
        println!("    --clear-memory             Remove memory content for current chat" );
        println!("    --select-histroy=<id>      Show fact by id from history" );
        println!("    --select-memory=<id>       Show fact by id from memory" );
        println!("    --delete-history=<id>      Delete fact by id from history" );
        println!("    --delete-memory=<id>       Delete fact by id from memory" );
        println!("    --update-history=<id>      Update fact by id in history" );
        println!("    --update-memory=<id>       Update fact by id in memory" );
        println!("    --insert-history=<id>      Insert new fact into history" );
        println!("    --insert-memory=<id>       Insert new fact into memory" );
        println!("    --actor=<actor>            Actor for insert/update (default: @ASSISTANT)");
        println!("    --body=<text>              Body for insert/update (or from stdin)");
        println!("");
        println!("Specific:");
        println!("    --write-pool               Write stdin to pool file and forward to stdout");
        println!("                               Example: echo 'data' | ai --write-pool");
        println!("    --tiocsti                  Inject input directly into TTY input buffer for keyboard");
        println!("                               Requires `sudo sysctl -w dev.tty.legacy_tiocsti=1`");
        println!("                               on modern kernels. Only use in trusted environments.");
        println!("                               Example: echo 'ls -la' | ai --tiocsti");
        println!("");
        println!("    --completion=<shell>       Generate shell completion (bash|zsh|fish)");
        println!("                               Example: ai --completion=bash >> ~/.bashrc");
        println!("");
        println!("Recommendations:");
        println!("    alias                      Set `alias 1=ai`");
        println!("");
        println!("Author:");
        println!("    Still Swamp (still@catlair.net) Powered by deepseek");
        self
    }



    /*
        Build yaml with session information and return it in to stdout
    */
    fn show_info( &mut self )
    -> &mut Self
    {
        let info = json!
        (
            {
                "log": self.get_app().get_log().get_file_path(),
                "config": self.get_config_file(),
                "version": self.get_version(),
                "session": 
                {
                    "profile": self.get_profile(),
                    "provider": self.get_provider(),
                    "chat": self.get_chat(),
                    "model": self.get_model(),
                    "proxy":  self.read_proxy()
                },
                "access":
                {
                    "history": self.history_storage.get_access(),
                    "memory": self.memory_storage.get_access()
                },
                "files":
                {
                    "prompt_chat": self.get_prompt_file( "chat" ),
                    "prompt_summary": self.get_prompt_file( "summary" ),
                    "model": self.get_model_file_path(),
                    "memory": self.get_memory_file(),
                    "token": self.get_token_path(),
                    "history": self.get_history_file_path()
                },
                "statistics":
                {
                    "max_prompt_size_bytes": self.get_max_chat_prompt_size_byte(),
                    "history_size_bytes": self.history_storage.to_string().len(),
                    "memory_size_bytes": self.memory_storage.to_string().len()
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
            self.switch_profile( &profile );
            no_prompt = true;
        }

        /* Set profile for current session */
        let profile = self.app.config[ "profile" ].get_str( "" );
        if !profile.is_empty()
        {
            self.set_profile( &profile );
        }
        else
        {
            self.read_profile();
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


            /* First log message */
            self.app.get_log_mut().begin
            (
                "=== Ai started =================================================="
            );
            self.app.dump_config();

            /*
                Main section
            */

            /* Check config */
            if self.app.state.is_ok()
            {
                /* Read current values */
                self.set_provider( &self.read_provider() );
                self.set_model( &self.read_model() );
                self.set_chat( &self.read_chat() );
                
                /* Set access rights */
                let cli_history = self.app.config[ "access-history" ].get_str("");
                let cli_memory = self.app.config[ "access-memory" ].get_str("");

                let default_history = self.app.config
                [ "application" ][ "ai" ][ "access" ][ "history" ].get_str("c");
                let default_memory = self.app.config
                [ "application" ][ "ai" ][ "access" ][ "memory" ].get_str("c");

                self.history_storage.set_access
                (
                    if !cli_history.is_empty() { &cli_history } else { &default_history }
                );

                self.memory_storage.set_access
                (
                    if !cli_memory.is_empty() { &cli_memory } else { &default_memory }
                );



                /* Collect actions from config into a map (copy values) */
                let mut actions = Vec::new();

                if let Some(mapping) = self.app.config.as_mapping()
                {
                    for (key, value) in mapping
                    {
                        let action = key.get_str("").to_string();
                        let target = value.get_str("").to_string();
                        actions.push((action, target));
                    }
                }

                /* Execute actions from collected map */
                for( action, target ) in &actions
                {
                    match action.as_str()
                    {
                        "m" | "model" => 
                        {
                            self.set_model( &target );
                        }


                        "p" | "provider" => 
                        {
                            self.set_provider( &target );
                        }


                        "c" | "chat" => 
                        {
                            self.set_chat( &target );
                        }
                        

                        "switch-model" => 
                        {
                            no_prompt = true;
                            self
                            .switch_model( &target )
                            .set_model( &target );
                        }


                        "switch-provider" => 
                        {
                            no_prompt = true;
                            self
                            .switch_provider( &target )
                            .set_provider( &target );
                        }


                        "switch-chat" => 
                        {
                            no_prompt = true;
                            self
                            .switch_chat( &target )
                            .set_chat( &target );
                        }

                        _ => {}
                    }
                }

                /* Open history */
                let history_path = self.get_history_file_path();
                self.history_storage
                .load( &history_path )
                .get_state()
                .state_to( &mut self.app.state );           

                /* Open memory */
                let memory_path = self.get_memory_file();
                self.memory_storage.
                load( &memory_path )
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
                            self.show_info();
                        }


                        
                        "history" | "show-history" => 
                        {
                            no_prompt = true;
                            self.show_history();
                        }


                        "memory" |  "show-memory" => 
                        {
                            no_prompt = true;
                            self.show_memory();
                        }


                        "prompt" | "show-prompt" => 
                        {
                            no_prompt = true;
                            let user_prompt = self.get_user_prompt();
                            let prompt = self.build_prompt( &user_prompt, "chat" );
                            println!( "{}", prompt );
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



                        "clear-history" => 
                        {
                            no_prompt = true;
                            self.clear_history();
                        }


                        "clear-memory" => 
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
                                print!( "{}", self.generate_completion( target ));
                            }
                        }

                        "select-history" =>
                        {
                            no_prompt = true;
                            if !target.is_empty()
                            {
                                let (actor, body) = self.history_storage.select( &target );
                                println!("{}\n\n{}", actor, body);
                            }
                        }

                        "select-memory" =>
                        {
                            no_prompt = true;
                            if !target.is_empty()
                            {
                                let (actor, body) = self.history_storage.select( &target );
                                println!("{}\n\n{}", actor, body);
                            }
                        }


                        "delete-history" => 
                        {
                            {
                                no_prompt = true;
                                if !target.is_empty()
                                {
                                    self.history_storage.delete( &target );
                                }
                            }
                        }


                        "delete-memory" => 
                        {
                            {
                                no_prompt = true;
                                if !target.is_empty()
                                {
                                    self.memory_storage.delete( &target );
                                }
                            }
                        }


                        "insert-history" =>
                        {
                            no_prompt = true;
                            let actor = self.app.config[ "actor" ].get_str("@USER");
                            let mut body = self.app.config[ "body" ].get_str( "" );
                            
                            if body.is_empty()
                            {
                                let mut input = String::new();
                                std::io::stdin().read_to_string(&mut input).ok();
                                body = input.trim().to_string();
                            }
                            
                            if !body.is_empty()
                            {
                                self.history_storage.create( &actor, &body );
                            }
                        }

                        "insert-memory" =>
                        {
                            no_prompt = true;
                            let actor = self.app.config[ "actor" ].get_str( "@USER" );
                            let mut body = self.app.config[ "body" ].get_str("");
                            
                            if body.is_empty()
                            {
                                let mut input = String::new();
                                std::io::stdin().read_to_string(&mut input).ok();
                                body = input.trim().to_string();
                            }
                            
                            if !body.is_empty()
                            {
                                self.memory_storage.create( &actor, &body );
                            }
                        }

                        "update-history" =>
                        {
                            no_prompt = true;
                            let actor = self.app.config[ "actor" ].get_str( "@USER" );
                            let mut body = self.app.config[ "body" ].get_str("");
                            
                            if !target.is_empty()
                            {
                                if body.is_empty()
                                {
                                    let mut input = String::new();
                                    std::io::stdin().read_to_string(&mut input).ok();
                                    body = input.trim().to_string();
                                }                               
                                if !body.is_empty()
                                {
                                    self.history_storage.update( &target, &actor, &body );
                                }
                            }
                        }

                        "update-memory" =>
                        {
                            no_prompt = true;
                            let actor = self.app.config[ "actor" ].get_str( "@USER" );
                            let mut body = self.app.config["body"].get_str("");
                            
                            if !target.is_empty()
                            {
                                if body.is_empty()
                                {
                                    let mut input = String::new();
                                    std::io::stdin().read_to_string(&mut input).ok();
                                    body = input.trim().to_string();
                                }
                                if !body.is_empty()
                                {
                                    self.memory_storage.update( &target, &actor, &body );
                                }
                            }
                        }
                        _ => {}
                    }
                }



                if !no_prompt
                {
                    let provider_name = self.get_provider();
                    let user_prompt = self.get_user_prompt();
                    let prompt = self.build_prompt( "chat", &user_prompt );

                    let max_bytes = self.get_max_chat_prompt_size_byte();
                    let size = prompt.len();
                    
                    if size > max_bytes
                    {
                        println!
                        (
                            "Prompt size {} bytes exceeds limit {} bytes.\n\
                             Please increase max-chat-prompt-size-byte in config,\n\
                             or run 'ai pack history --allow-history=cud' to compress conversation history.",
                            size, max_bytes
                        );
                    }
                    else
                    {
                        /* Write user prompt to history */
                        self.history_storage.create( "@USER", &user_prompt );

                        let mut provider = providers::create_provider(&provider_name, self);
                        provider.chat(&prompt);
                    }
                }            
            }

            /* Save current state */
            let history_path = self.get_history_file_path();
            self.history_storage.save( &history_path );
            /* Save current state */
            let memory_path = self.get_memory_file();
            self.memory_storage.save( &memory_path );

            self.app.get_log_mut().end( "End of ai" ).eol();
        }
        
        self
    }



    /**************************************************************************
        Prompt Building
    */

    /*
        Return prompt file path for chat
        Uses global prompts section with placeholders
            %profile%,
            %provider%,
            %chat%,
            %model%
    */
    fn get_prompt_file
    (
        &self, 
        /* chat | summary */
        prompt_type: &str
    )
    /* Return prompt file name */
    -> String
    {
        core::expand_path
        (
            &self.get_config_val
            (
                &[ "prompts", prompt_type ],
                format!
                (
                    "~/.config/ai/app/cli/%profile%/prompts/%provider%/%model%/{}.txt", 
                    prompt_type
                )
            )
            .replace( "%profile%", &self.get_profile() )
            .replace( "%provider%", &self.get_provider() )
            .replace( "%model%", &self.get_model_safe() )
            .replace( "%chat%", &self.get_chat() )
        )
    }


         
    /*
        Read prompt template from file
    */
    fn read_prompt
    (
        &mut self, 
        prompt_type: &str
    )
    -> String
    {
        let full_path = self.get_prompt_file( prompt_type );
        
        /* If file doesn't exist, create it from binary */
        if !std::path::Path::new(&full_path).exists()
        {
            /* Ensure parent directory exists */
            if let Some( parent ) = std::path::Path::new( &full_path ).parent()
            {
                if let Err( e ) = std::fs::create_dir_all( parent )
                {
                    self.app.get_log_mut()
                    .error( "Failed to create prompt directory" )
                    .prm( "path", &parent.to_string_lossy() )
                    .prm( "error", &e );
                    return match prompt_type
                    {
                        "chat" => prompts::CHAT.to_string(),
                        _ => "%user-prompt%".to_string(),
                    };
                }
            }
            
            let default_content = match prompt_type
            {
                "chat" => prompts::CHAT,
                _ => "",
            };
            
            if let Err(e) = std::fs::write(&full_path, default_content)
            {
                self.app.get_log_mut()
                .error("Failed to create prompt file")
                .prm("path", &full_path)
                .prm("error", &e);
                return default_content.to_string();
            }
            
            self.app.get_log_mut()
            .info("Default prompt created")
            .prm("path", &full_path)
            .prm("type", prompt_type);
        }
        
        /* Read prompt file */
        match std::fs::read_to_string(&full_path)
        {
            Ok( content ) => content,
            Err( e ) =>
            {
                self.app.get_log_mut()
                .error( "Failed to read prompt file" )
                .prm( "path", &full_path )
                .prm( "error", &e );
                
                match prompt_type
                {
                    "chat" => prompts::CHAT.to_string(),
                    _ => "%user-prompt%".to_string(),
                }
            }
        }
    }


    /*
        Return user prompt combining stdin pipe, CLI arguments, and interactive input.

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
            match stdin.read_to_string(&mut pool)
            {
                Ok(0) => 
                {
                    /* Pipe exists but empty - do nothing, prompt stays empty */
                }
                Ok(_) => 
                {
                    prompt.push_str(pool.trim());
                }
                Err(e) => 
                {
                    self.app.get_log_mut()
                        .error("Failed to read from stdin pipe")
                        .prm("error", &e.to_string());
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
            println!( "Enter your prompt (Ctrl+D to finish or Ctrl+C to cancel):" );
            
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
        prompt_type: &str, 
        /* User prompt */
        input: &str
    )
    -> String 
    {
        let template = self.read_prompt( prompt_type );

        /* Retrive shell */
        let shell = self.get_config_val
        (
            &[ "shell" ], 
            "/bin/bash".to_string()
        );

        let input = input
        .replace( BLOCK_DELIMITER, "<block-delimiter>" )
        .replace( "%block-delimiter%", "<block-delimiter>" );

        let result = template
        .replace( "%history%", &self.get_history() )
        .replace( "%memory%", &self.read_memory() )
        .replace( "%user-prompt%", &input )
        .replace( "%shell%", &shell )
        .replace( "%chat%", &self.get_chat() )
        .replace( "%provider%", &self.get_provider() )
        .replace( "%model%", &self.get_model() )
        .replace
        (
            "%now%", 
            &Moment::create().now().format( "%Y-%m-%d %H:%M:%S" )
        )
        .replace( "%block-delimiter%", BLOCK_DELIMITER )
        ;

        result
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



    fn get_history( &self )
    -> String 
    {
        let history_path = self.get_history_file_path();       
        match std::fs::read_to_string(&history_path) 
        {
            Ok( content ) => content,
            Err(_) => String::new(),
        }
    }



    /*
        Clear history file
    */
    fn clear_history(&mut self) -> &mut Self
    {
        /* Clear all blocks from memory */
        self.history_storage.clear();
        self.app.get_log_mut().info( "History cleared" );

        self
    }


    /*
        Send history to stdout
    */
    fn show_history( &mut self )
    -> &mut Self 
    {
        let history = self.get_history();
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




    /*************************************************************************
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
        core::expand_path("~/.config/ai/app/cli/%profile%/config.yaml")
        .replace("%profile%", &self.get_profile())
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
            .replace( "%provider%", &self.get_provider())
            .replace( "%model%", &self.get_model_safe())
            .replace( "%chat%", &self.get_chat())
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
        let default = "~/.local/share/ai/app/cli/%profile%/models/%provider%.txt".to_string();
        let path = self.get_config_val( &[ "model" ], default );
        
        core::expand_path
        (
            &path
            .replace( "%profile%", &self.get_profile() )
            .replace( "%provider%", &self.get_provider() )
            .replace( "%chat%", &self.get_chat() )
        )
    }



    pub fn read_model(&self) -> String
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

        let available: Vec<String> = self.get_config_val
        (
            &["available-models"],
            vec![]
        );

        available.first().cloned().unwrap_or_else(|| "gpt-4.1".to_string())
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
            .trace("Model switched")
            .prm("id", id);
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
    fn set_provider(&mut self, name: &str) -> &mut Self
    {
        self.provider = name.to_string();
        self
    }



    /*
        Return provider file path
    */
    fn get_provider_file_path(&self) -> String
    {
        let path = self.get_config_val
        (
            &[ "provider_file" ],
            "~/.config/ai/app/cli/%profile%/provider.txt".to_string()
        );
        
        core::expand_path( &path.replace( "%profile%", &self.get_profile() ))
    }


    /*
        Return provider
    */
    fn read_provider( &self )
    -> String
    {
        let path = self.get_provider_file_path();

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
        let file_path = self.get_provider_file_path();

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
    
        self.set_profile(&profile);
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
            .error("Failed to write profile")
            .prm("path", &path)
            .prm("error", &e.to_string());
        } 
        else
        {
            self.app.get_log_mut()
            .trace( "Profile saved" )
            .prm("name", name);
        }
        
        self
    }



    /*
        Switch profile 
    */
    fn switch_profile
    (
        &mut self,
        /* Profile name */ 
        name: &str
    )
    -> &mut Self 
    {
        self.write_profile( name );
        self.profile = name.to_string();
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
            .replace( "%model%", &self.get_model_safe() )
            .replace( "%provider%", &self.get_provider() )
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

        if let Ok(content) = std::fs::read_to_string( &path ) 
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
            .error("Failed to ensure chat directory")
            .prm("error", &e);
            return self;
        }
        
        if let Err(e) = std::fs::write(&file_path, new_id)
        {
            self.app.get_log_mut()
            .error("Failed to switch chat")
            .prm("path", &file_path)
            .prm("error", &e.to_string());
        } 
        else
        {
            self.app.get_log_mut()
            .trace("Chat switched")
            .prm("id", new_id);
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

        match std::process::Command::new
        (
            shell
        )
            .arg( "-c" )
            .arg( command )
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
                            .prm( "exit_code", exit_status.code().unwrap_or( -1 ));
                        }
                        Err( e ) =>
                        {
                            self.app.get_log_mut()
                            .warning("Failed to wait for command")
                            .prm("command", command)
                            .prm("error", &e.to_string());
                        }
                    }
                }
                else 
                {
                    self.app.get_log_mut()
                    .info("Command spawned (no wait)")
                    .prm("command", command)
                    .prm("data_bytes", data_len);
                    
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
                .prm( "command", command )
                .prm( "data_bytes", data.len() )
                .prm( "error", &e.to_string() );
                println!( "{}", data );
            }
        }
    }



    /*
        Processing chat response
    */
    pub fn handle_chat_response
    (
        &mut self,
        response: &serde_json::Value
    )
    {
        /* Extract fields from JSON */
        let pool_path = self.get_pool_path();
        let message = response["message"].get_str( "" ).replace("%pool%", &pool_path);
        let command = response["command"].get_str( "" ).replace("%pool%", &pool_path);
        let pool = response["pool"].get_str( "" ).replace("%pool%", &pool_path);
        let clipboard = response[ "clipboard" ].get_str("").replace("%pool%", &pool_path);

        /* Write to history if has content */
        if !message.is_empty() || !command.is_empty()
        {
            self.history_storage.create
            (
                "@ASSISTANT",
                &format!( "{}\n\n{}", message, command )
            );
        }

        /* Handle memory operations */
        /* Add new entries */
        let add = response[ "memory" ]["add"].get_array(vec![]);
        for item in add
        {
            let text = item.get_str( "" );
            if !text.is_empty()
            {
                self.memory_storage.create( "@ASSISTANT", &text );
                self.app.get_log_mut()
                .info( "Memory entry added" )
                .prm( "text", &text );
            }
        }
        
        /* 
            Remove entries by ID
        */
        let remove = response[ "memory" ][ "remove" ].get_array(vec![]);
        for item in remove
        {
            let id = item.get_str("");
            if !id.is_empty()
            {
                self.memory_storage.delete(&id);
                self.app.get_log_mut()
                .info( "Memory entry removed" )
                .prm( "id", &id );
            }
        }
        
        /* Change entries by ID */
        let change = response[ "memory" ][ "change" ].get_array(vec![]);
        for item in change
        {
            let id = item[ "id" ].get_str( "" );
            let actor = item[ "actor" ].get_str( "@USER" );
            let body = item[ "body" ].get_str( "" );
            
            if !id.is_empty() && !body.is_empty()
            {
                self.memory_storage.update( &id, &actor, &body );
                self.app.get_log_mut()
                .info( "Memory entry changed" )
                .prm( "id", &id )
                .prm( "actor", &actor )
                .prm( "new_body", &body );
            }
        }



        /* 
            Handle history operations
        */
        /* Add new entries */
        let add = response[ "history" ][ "add" ].get_array( vec![] );
        for item in add
        {
            let text = item.get_str( "" );
            if !text.is_empty()
            {
                self.history_storage.create( "@ASSISTANT", &text );
                self.app.get_log_mut()
                    .info( "History entry added" )
                    .prm( "text", &text );
            }
        }

        /* Remove entries by ID */
        let remove = response[ "history" ][ "remove" ].get_array( vec![] );
        for item in remove
        {
            let id = item.get_str( "" );
            if !id.is_empty()
            {
                self.history_storage.delete(&id);
                self.app.get_log_mut()
                .info( "History entry removed" )
                .prm("id", &id);
            }
        }

        /* Change entries by ID */
        let change = response[ "history" ][ "change" ].get_array(vec![]);
        for item in change
        {
            let id = item[ "id" ].get_str( "" );
            let actor = item[ "actor" ].get_str( "@ASSISTANT" );
            let body = item[ "body" ].get_str( "" );
            
            if !id.is_empty() && !body.is_empty()
            {
                self.history_storage.update( &id, &actor, &body );
                self.app.get_log_mut()
                .info( "History entry changed" )
                .prm( "id", &id)
                .prm( "actor", &actor)
                .prm( "new_body", &body);
            }
        }



        /* Output message via destination */
        if !message.is_empty()
        {
            self.run_destination( &message, "message", true );
        }

        /* Put information to clipboard */
        if !clipboard.is_empty()
        {
            self.run_destination( &clipboard, "clipboard", true );
        }

        /* Execute command via destination */
        if !command.is_empty()
        {
            /* Check if command execution is disabled */
            if self.app.config[ "no-command" ].get_bool( false )
            {
                self.app.get_log_mut()
                .info( "Command execution disabled by --no-command" )
                .prm( "command", &command );
            }
            else
            {
                /* 
                    REMOVE_ENTER

                    Removes newline and carriage return characters from LLM-generated command.
                    Prevents command injection via line breaks that could:
                    1. Terminate the current command
                    2. Inject arbitrary new commands
                    3. Execute hidden malicious code

                    The cleaned command remains as a single line.
                    Only newline/carriage return are removed - all other characters (&&, |, ;, $, `, etc.)
                    are preserved as legitimate command syntax.
                */
                let clean_command = command.replace( '\n', " ").replace('\r', "");
                self.run_destination( &clean_command, "command", false );
            }
        }

        /* Write pool via destination */
        if !pool.is_empty()
        {
            self.run_destination( &pool, "pool", true );
        }

        /* Log token usage */
        self.app.get_log_mut()
        .trace("Token usage")
        .prm("prompt_tokens", response["prompt_tokens"].as_u64().unwrap_or(0))
        .prm("answer_tokens", response["answer_tokens"].as_u64().unwrap_or(0));
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
                        .prm( "error", &std::io::Error::last_os_error().to_string());

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
                "~/.local/share/ai/app/cli/%profile%/memory/%chat%.txt".to_string()
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
        /* Clear all blocks from memory */
        self.memory_storage.clear();
        self.app.get_log_mut().info( "Memory cleared" );
        self
    }



    /*
        Read memory for current chat
    */
    fn read_memory( &self )
    -> String
    {
        let memory_path = self.get_memory_file();       
        match std::fs::read_to_string( &memory_path )
        {
            Ok( content ) => content,
            Err(_) => String::new(),
        }
    }



    /*
        Send memory to stdout
    */
    fn show_memory( &mut self )
    -> &mut Self 
    {
        let memory = self.read_memory();
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
        /* Provider name (e.g., "github", "openai", "deepseek") */
        provider: &str,
        /* Model identifier (e.g., "gpt-4.1", "deepseek-chat", "claude-3-5-sonnet") */
        model: &str,
        /* API endpoint URL for the request */
        api_url: &str,
        /* Type of request: "chat" for conversation, "summary" for history compression */
        prompt_type: &str
    )
    {
        self.app.get_log_mut()
        .dump( "Prompt to LLM", prompt )
        .prm( "provider", provider )
        .prm( "model", model )
        .prm( "api", api_url )
        .prm( "type", prompt_type );
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
        /* Provider name (e.g., "github", "openai", "deepseek") */
        provider: &str,
        /* Model identifier used for the request */
        model: &str,
        /* API endpoint URL used for the request */
        api_url: &str,
        /* Type of request: "chat" or "summary" */
        prompt_type: &str
    )
    {
        self.app.get_log_mut()
        .dump( "Response from LLM", response )
        .prm( "provider", provider )
        .prm( "model", model )
        .prm( "api", api_url )
        .prm( "type", prompt_type );
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
            "--help",
            "--info",
            "--version",
            "--help",
            "--info",
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

            // LLM access rights (CUD)
            "--access-history=",
            "--access-memory=",

            // Storage operations: history
            "--history",
            "--clear-history",
            "--select-history=",
            "--delete-history=",
            "--update-history=",
            "--insert-history=",

            // Storage operations: memory
            "--memory",
            "--clear-memory",
            "--select-memory=",
            "--delete-memory=",
            "--update-memory=",
            "--insert-memory=",

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
        let options_str = options.join(" ");
        
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
            &format!("    COMPREPLY=($(compgen -W '{}' -- \"$cur\"))", options),
            "}",
            "complete -F _ai ai",
            "complete -F _ai 1\n",
        ]
        .join("\n")
    }



    /*
        Generate zsh completion
    */
    fn generate_zsh_completion
    (
        &self, 
        options: &[&str]
    )
    -> String
    {
        let args = options.iter()
            .map(|o| format!("  '{}'", o))
            .collect::<Vec<_>>()
            .join(" \\\n");
        
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
        .join("\n")
    }



    /*
        Generate fish completion
    */
    fn generate_fish_completion
    (
        &self, options: &[&str]
    ) -> String
    {
        let mut fish = String::new();
        for opt in options
        {
            let opt_clean = opt.trim_end_matches('=');
            fish.push_str( &format!( "complete -c ai -f -a '{}'\n", opt_clean ));
            if opt.ends_with('=')
            {
                fish.push_str( &format!( "complete -c ai -f -a '{}<'\n", opt_clean ));
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



    /*
        Parse LLM response in block format
        Splits response by BLOCK_DELIMITER, extracts block name and content,
        then executes corresponding actions (message, command, history, memory, etc.)
    */
    pub fn parse_llm_response
    (
        &mut self, 
        content: &str, 
        response_json: &mut serde_json::Value
    )
    {
        self.get_app_mut().get_log_mut().dump( "llm content", content );
        let block_delimiter = BLOCK_DELIMITER;
        let blocks: Vec<&str> = content.split( block_delimiter ).collect();
        for block in blocks
        {
            let block = block.trim();
            if !block.is_empty()
            {               
                let lines: Vec<&str> = block.lines().collect();
                if !lines.is_empty()
                {
                    let block_name = lines[0].trim();
                    let block_content = &lines[1..].join( "\n" );
                    match block_name
                    {
                        "message" =>
                        {
                            response_json[ "message" ] = serde_json::json!
                            (
                                block_content
                            );
                        }
                        "command" =>
                        {
                            response_json[ "command" ] = serde_json::json!
                            (
                                block_content
                            );
                        }
                        "pool" =>
                        {
                            response_json["pool"] = serde_json::json!
                            (
                                block_content
                            );
                        }

                        "clipboard" =>
                        {
                            response_json[ "clipboard" ] = serde_json::json!
                            (
                                block_content
                            );
                        }
                        
                        "history-add" =>
                        {
                            if response_json[ "history" ][ "add" ].is_null()
                            {
                                response_json
                                [ "history" ]
                                [ "add" ] = serde_json::json!([]);
                            }

                            response_json[ "history" ][ "add" ]
                            .as_array_mut()
                            .unwrap()
                            .push(serde_json::json!( block_content ));
                        }

                        "history-remove" =>
                        {
                            if response_json[ "history" ][ "remove" ].is_null()
                            {
                                response_json
                                [ "history" ]
                                [ "remove" ] = serde_json::json!([]);
                            }
                            response_json[ "history" ][ "remove" ]
                            .as_array_mut()
                            .unwrap()
                            .push
                            (
                                serde_json::json!
                                (
                                    block_content.trim().to_string()
                                ) 
                            );
                        }

                        "history-change" =>
                        {
                            if response_json[ "history" ][ "change" ].is_null()
                            {
                                response_json[ "history" ][ "change" ] = serde_json::json!([]);
                            }
                            
                            let lines: Vec<&str> = block_content.lines().collect();

                            if lines.len() >= 3
                            {
                                let id = lines[0].trim();
                                let actor = lines[1].trim();
                                let body = lines[2..].join("\n").trim().to_string();
                                
                                response_json[ "history" ][ "change" ]
                                .as_array_mut()
                                .unwrap()
                                .push
                                (
                                    serde_json::json!
                                    (
                                        {
                                            "id": id,
                                            "actor": actor,
                                            "body": body
                                        }
                                    )
                                );
                            }
                        }

                        "memory-add" =>
                        {
                            if response_json[ "memory" ][ "add" ].is_null()
                            {
                                response_json
                                [ "memory" ]
                                [ "add" ] = serde_json::json!([]);
                            }

                            response_json[ "memory" ][ "add" ]
                            .as_array_mut()
                            .unwrap()
                            .push(serde_json::json!(block_content));
                        }

                        "memory-remove" =>
                        {
                            if response_json[ "memory" ][ "remove" ].is_null()
                            {
                                response_json
                                [ "memory" ]
                                [ "remove" ] = serde_json::json!([]);
                            }
                            response_json[ "memory" ][ "remove" ]
                            .as_array_mut()
                            .unwrap()
                            .push
                            (
                                serde_json::json!
                                (
                                    block_content.trim().to_string()
                                ) 
                            );
                        }

                        "memory-change" =>
                        {
                            if response_json["memory"]["change"].is_null()
                            {
                                response_json
                                ["memory"]
                                ["change"] = serde_json::json!([]);
                            }

                            let lines: Vec<&str> = block_content.lines().collect();

                            if lines.len() >= 3
                            {
                                let key = lines[0].trim();
                                let actor = lines[1].trim();
                                let body = lines[2..].join("\n").trim().to_string();

                                response_json["memory"]["change"]
                                .as_array_mut()
                                .unwrap()
                                .push
                                (
                                    serde_json::json!
                                    (
                                        {
                                            "id": key,
                                            "actor": actor,
                                            "body": body
                                        }
                                    )
                                );
                            }
                        }

                        "end" => { break; }
                        _ => {}
                    }
                }
            }
        }
    }

}
