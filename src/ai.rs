/*
    SPDX-License-Identifier: MIT
    SPDX-FileCopyrightText: 2026 Still Swamp
*/

/*
    Main AI module
*/

#[path = "providers/mod.rs"]
mod providers;

#[path = "response.rs"]
mod response;

use core::Moment;
use core::{ expand_path, ensure_directory };

use serde_json;
use serde_yaml::Value;
use core::State;
use core::Application;
use std::io::Read;
use std::io::Write;
use std::io::IsTerminal;
use crate::ai::response::ChatResponse;



/*
    Ai applicatoin
*/
pub struct Ai 
{
    /* Application structure */
    pub application: Application,

    /* Ai state structure */
    state: State,

    /* Profile for current session */
    profile: String,

    /* AI provider for current session */
    provider: String,

    /* Chat id */
    chat: String,

    /* Id of model of provider for current session */
    model: String
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
            state: State::ok(),
            application: Application::create(),
            profile: "default".to_string(),
            provider: "github".to_string(),
            model: String::new(),
            chat: "default".to_string(),
        }
    }



    /*
        Help utility
    */
    fn help( &mut self )
    -> &mut Self
    {
        println!( "{}\n", self.get_version());
        println!( "" );
        println!( "Usage:" );
        println!( "    ai                         Interactive keyboard input" );
        println!( "    ai <question>              Ask a question" );
        println!( "    echo <text> | ai           Read from stdin" );
        println!( "    ai --help                  Show this help" );
        println!( "" );
        println!( "Options:" );
        println!( "    --help                     This information" );
        println!( "    --info                     Show current runtime information" );
        println!( "    --version                  Show current version" );
        println!( "    --no-prompt                Suppress input user prompt" );
        println!( "    --no-command               Suppress command event " );
        println!( "" );
        println!( "    --provider=<name>          Use provider for current session only (no save)" );
        println!( "    --switch-provider=<name>   Switch to AI provider <name> (saves to file)" );
        println!( "    --profile=<name>           Use profile for current session only" );
        println!( "    --switch-profile=<name>    Switch and save profile" );
        println!( "    --model=<name>             Use model for current session only (no save)" );
        println!( "    --switch-model=<name>      Switch and save model" );
        println!( "    --chat=<id>                Switch to chat <id> for current session only (no save)" );
        println!( "    --switch-chat=<id>         Switch to chat <id> (saves to file)" );
        println!( "" );
        println!( "    --show-history             Show history for current chat" );
        println!( "    --clear-history            Remove history for current chat" );
        println!( "    --pack-history=<percent>   Pack current chat history with 0-100 percent (default: 50)" );
        println!( "    --show-memory              Show memory for current chat" );
        println!( "    --clear-memory             Remove memory for current chat (global if %chat% not used)" );
        println!( "" );
        println!( "    --write-pool               Write stdin to pool file and forward to stdout" );
        println!( "                               Example: echo 'data' | ai --write-pool");
        println!( "    --tiocsti                  Inject input directly into TTY input pool for keyboard" );
        println!( "                               Requires `sudo sysctl -w dev.tty.legacy_tiocsti=1`" );
        println!( "                               on modern kernels. Only use in trusted environments." );
        println!( "                               Example: echo 'ls -la' | ai --tiocsti" );
        println!( "" );
        println!( "Recommendations:" );
        println!( "    alias                      Set `alias 1=ai`" );
        println!( "" );
        println!( "Author:" );
        println!( "    Still Swamp (still@catlair.net) Build with deepseek" );

        self
    }


    /*
        Build yaml with session information and return it in to stdout
    */
    fn show_info( &mut self )
    -> &mut Self
    {
        let info = serde_json::json!
        (
            {
                "log": self.application.get_log().get_file_path(),
                "config": self.get_config_file(),
                "version": self.get_version(),
                "session": 
                {
                    "profile": self.get_profile(),
                    "provider": self.get_provider(),
                    "chat": self.get_chat(),
                    "model": self.get_model()
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
                    "history_size_bytes": self.get_history_size_byte(),
                    "memory_size_bytes": self.get_memory_size_byte()
                }
            }
        );
        
        println!("{}", serde_yaml::to_string(&info).unwrap_or_default());
        
        self
    }


    /*
        Main run method
    */
    pub fn run ( &mut self ) 
    -> &mut Self 
    {
        /* Read cli arguments */
        self.application.read_cli();

        /*
            Profile section
        */

        /* Set profile */
        if let Some( profile ) = self.application.config
            .as_ref()
            .and_then( |cfg| cfg[ "switch-profile" ].as_str() )
        {
            self.switch_profile( &profile.to_string() );
        }

        /* Set profile for current session */
        if let Some( profile ) = self.application.config
        .as_ref()
        .and_then(|cfg| cfg["profile"].as_str())
        {
            self.set_profile( &profile.to_string() );
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
        self.application.read_config( &path ).read_cli();

        /*
            Log section
        */

        /* Set log file */
        if let Some(file) = self.application.config.as_ref()
        .and_then(|c| c["application"]["log"]["file"].as_str())
        {
            let file = expand_path(file).replace( "%profile%", &self.get_profile() );
            self.application.get_log().set_file_path(&file);
        }

        /* First log message */
        self.application.get_log().begin
        (
            "=== Ai started =================================================="
        );
        self.application.dump_config();


        /*
            Main section
        */

        /* Check config */
        if self.application.state.is_ok()
        {
            /* No prompt mode */
            let mut no_prompt = self.application.config
                .as_ref()
                .and_then(|cfg| cfg["no-prompt"].as_bool())
                .unwrap_or( false );

            /* Switch profile mode */
            if let Some(_) = self.application.config.as_ref()
                .and_then(|cfg| cfg["switch-profile"].as_str())
            {
                no_prompt = true;
            }



            /* Set provider */
            if let Some( profile ) = self.application.config
                .as_ref()
                .and_then(|cfg| cfg[ "switch-provider" ].as_str())
            {
                no_prompt = true;
                self.switch_provider( &profile.to_string() );
            }

            /* Set profile for current session */
            if let Some( profile ) = self.application.config
                .as_ref()
                .and_then(|cfg| cfg[ "provider" ].as_str())
            {
                self.set_provider( &profile.to_string() );
            }
            else
            {
                self.set_provider( &self.read_provider());
            }



            /* Switch chat */
            if let Some( chat ) = self.application.config
                .as_ref()
                .and_then(|cfg| cfg[ "switch-chat" ].as_str())
            {
                no_prompt = true;
                self.switch_chat( &chat.to_string() );
            }
            /* Set profile for current session */
            if let Some( chat ) = self.application.config
                .as_ref()
                .and_then(|cfg| cfg[ "chat" ].as_str())
            {
                self.set_chat( &chat.to_string() );
            }
            else
            {
                self.set_chat( &self.read_chat());
            }



            /* Switch model */
            if let Some( model ) = self.application.config
                .as_ref()
                .and_then(|cfg| cfg[ "switch-model" ].as_str())
            {
                no_prompt = true;
                self.switch_model( &model.to_string() );
            }



            /* Set model */
            if let Some( model ) = self.application.config
                .as_ref()
                .and_then( |cfg| cfg[ "model" ].as_str() )
            {
                self.set_model( &model.to_string() );
            }
            else
            {
                self.set_model( &self.read_model() );
            }



            /* Help mode */
            if let Some(_) = self.application.config
                .as_ref()
                .and_then(|cfg| cfg["help"].as_bool())
            {
                no_prompt = true;
                self.help();
            }



            /* Help mode */
            if let Some(_) = self.application.config
                .as_ref()
                .and_then(|cfg| cfg["version"].as_bool())
            {
                no_prompt = true;
                println!( "{}\n", self.get_version() );
            }


            /* Check clear-history flag */
            if let Some(_) = self.application.config.as_ref()
                .and_then(|cfg| cfg["clear-history"].as_bool())
            {
                no_prompt = true;
                self.clear_history();
            }

            /* Check clear-memory flag */
            if let Some(true) = self.application.config.as_ref()
                .and_then(|cfg| cfg["clear-memory"].as_bool())
            {
                no_prompt = true;
                self.clear_memory();
            }

            /* Check write-pool flag */
            if let Some(true) = self.application.config.as_ref()
                .and_then(|cfg| cfg["write-pool"].as_bool())
            {
                no_prompt = true;
                
                let mut input = String::new();
                if let Ok(_) = std::io::stdin().read_to_string(&mut input) {
                    self.write_pool( &input ); 
                }
            }

            /* Check tiocsti flag */
            if let Some(true) = self.application.config.as_ref()
                .and_then(|cfg| cfg["tiocsti"].as_bool())
            {
                no_prompt = true;
                /* Read from stdin */
                let mut input = String::new();
                match std::io::stdin().read_to_string(&mut input)
                {
                    Ok( 0 ) => 
                    {
                        self.application.get_log()
                            .warning("tiocsti: stdin is empty");
                    }
                    Ok(_) =>
                    {
                        self.input_tiocsti( &input );
                    }
                    Err( e ) =>
                    {
                        self.application.get_log()
                            .error("tiocsti: failed to read stdin")
                            .prm("error", &e.to_string());
                    }
                }
            }

            /* History request */
            if let Some(_) = self.application.config.as_ref()
                .and_then(|cfg| cfg["show-history"].as_bool())
            {
                no_prompt = true;
                self.show_history();
            }

            /* Memory request */
            if let Some(_) = self.application.config.as_ref()
                .and_then(|cfg| cfg[ "show-memory" ].as_bool())
            {
                no_prompt = true;
                self.show_memory();
            }

            /* Pack history */
            if let Some(level_str) = self.application.config.as_ref()
            .and_then(|cfg| cfg["pack-history"].as_str())
            {
                if let Ok(level) = level_str.parse::<u64>() 
                {
                    no_prompt = true;
                    let provider_name = self.get_provider();
                    let mut provider = providers::create_provider(&provider_name, self);
                    provider.summary( level );
                }
            }

            /* Check dump-prompt flag */
            if let Some( true ) = self.application.config.as_ref()
                .and_then(|cfg| cfg[ "dump-prompt" ].as_bool())
            {
                no_prompt = true;
                let user_prompt = self.get_user_prompt();
                let prompt = self.build_prompt( &user_prompt, "chat" );
                println!( "{}", prompt );
            }

            if let Some(_) = self.application.config.as_ref()
                .and_then( |cfg| cfg[ "info" ].as_bool() )
            {
                no_prompt = true;
                self.show_info();
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
                         or run 'ai --pack-history' to compress conversation history.",
                        size, max_bytes
                    );
                }
                else
                {
                    /* Write user prompt to history */
                    self.write_history(self.get_history_delimiter(), "");
                    self.write_history("@USER", &user_prompt);

                    let mut provider = providers::create_provider(&provider_name, self);
                    provider.chat(&prompt);
                }
            }            
        }

        self.application.get_log().end( "End of ai" ).eol();

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
        expand_path
        (
            &self.get_config_val
            (
                &[ "prompts", prompt_type ],
                format!( "~/.config/ai/%profile%/prompts/%provider%/%model%/{}.txt", prompt_type )
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
        /* chat | summary */
        prompt_type: &str       
        
    ) -> String
    {
        let default_prompt = "%user-prompt%".to_string();
        let full_path = self.get_prompt_file( prompt_type );

        match std::fs::read_to_string(&full_path)
        {
            Ok(content) => content,
            Err(_) =>
            {
                self.application.get_log()
                    .warning("Cannot read prompt file, using default")
                    .prm("path", full_path);
                default_prompt
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
                    self.application.get_log()
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

        let result = template
        .replace( "%history%", &self.get_history() )
        .replace( "%memory%", &self.read_memory() )
        .replace( "%user-prompt%", input )
        .replace( "%shell%", &shell )
        .replace( "%chat%", &self.get_chat() )
        .replace( "%provider%", &self.get_provider() )
        .replace( "%model%", &self.get_model() )
        .replace( "%history-delimiter%", &self.get_history_delimiter() )
        .replace
        (
            "%now%", 
            &Moment::create().now().format( "%Y-%m-%d %H:%M:%S" )
        );

        result
    }



    /**************************************************************************
        History
    */


    /*
        Return history chat delimiter
    */
    pub fn get_history_delimiter( &self ) 
    -> &'static str
    {
        "=AIOL9B1MZX="
    }



    /*
        Return current history file size in bytes
    */
    fn get_history_size_byte( &self )
    /* History size in bytes */
    -> usize
    {
        let path = self.get_history_file_path();
        std::fs::metadata( &path )
        .map(|m| m.len() as usize)
        .unwrap_or(0)
    }



    fn get_history_file_path( &self )
    -> String
    {
        expand_path
        (
            &self.get_config_val
            (
                &[ "history" ],
                "~/.config/ai/%profile%/history/%chat%.txt".to_string()
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
        Write history
    */
    fn write_history
    (
        &mut self,
        role: &str,
        text: &str
    )
    {
        let history_path = self.get_history_file_path();
        
        /* Check directory exists */
        if let Err( e ) = ensure_directory( &history_path )
        {
            self.application.get_log()
            .error( "Failed to ensure history directory" )
            .prm( "error", &e );

            return;
        }
        
        use std::fs::OpenOptions;
        use std::io::Write;

        let now = Moment::create().now().format( "%Y-%m-%d %H:%M:%S" );
        let line = format!( "{}\nUTC {}\n{}\n\n", role, now, text );        
        
        if let Ok( mut file ) = OpenOptions::new()
            .create( true )
            .append( true )
            .open( &history_path )
        {
            let _ = file.write_all( line.as_bytes() );
        }
    }



    /*
        Clear history file
    */
    fn clear_history( &mut self )
    -> &mut Self
    {
        let history_path = self.get_history_file_path();      
        match std::fs::write( &history_path, "" )
        {
            Ok( _ ) =>
            {
                self.application.get_log()
                .info( "History cleared" )
                .prm("path", &history_path);
            }
            Err( e ) =>
            {
                self.application.get_log()
                .warning( "Failed to clear history" )
                .prm( "path", &history_path )
                .prm( "error", &e.to_string() );
            }
        }
        
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



    pub fn handle_summary_response
    (
        &mut self, 
        summary: &str,
        error: &str,
        think: &str, 
        recent_history: &str, 
        prompt_tokens: u64, 
        answer_tokens: u64, 
        success: bool, 
        old_blocks: usize, 
        kept_blocks: usize
    )
    {
        if success && !summary.is_empty()
        {
            let new_history = format!
            (
                "{}\n\n{}\n\n{}{}", 
                "@SUMMARY",
                summary, 
                self.get_history_delimiter(), 
                recent_history
            );

            std::fs::write(&self.get_history_file_path(), new_history).unwrap();

            self.application.get_log()
                .info( "History packed successfully" )
                .prm( "old_blocks", old_blocks )
                .prm( "kept_blocks", kept_blocks )
                .prm( "prompt_tokens", prompt_tokens )
                .prm( "answer_tokens", answer_tokens );

            if !think.is_empty()
            {
                self.application.get_log()
                    .trace("Summary think")
                    .prm("content", think);
            }
        }
        else
        {
            /* Output message via destination */
            if !error.is_empty()
            {
                self.run_destination( &error, "message" );
            }

            self.application.get_log().warning
            (
                "Failed to get summary, history unchanged"
            );
        }
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
        expand_path
        (
            &self.get_config_val
            (
                &[ "pool" ],
                "~/.local/share/ai/%profile%/pool.txt".to_string()
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
                self.application.get_log()
                    .info( "pool written to file" )
                    .prm( "path", &pool_path );
            }
            Err(e) => 
            {
                self.application.get_log()
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
        expand_path("~/.config/ai/%profile%/config.yaml")
        .replace("%profile%", &self.get_profile())
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
        /* Key path after "application.ai" */
        keys: &[&str],
        /* Default value */
        default: T,
    )
    -> T 
    {
        let config = match &self.application.config 
        {
            Some(c) => c,
            None => return default,
        };
        
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
            &ai_cfg["providers"][&provider]["models"][&model]["chats"][&chat]
        )
        {
            if let Ok(v) = serde_yaml::from_value(val)
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
        let default = "~/.config/ai/%profile%/tokens/%provider%.txt".to_string();
        let path = self.get_config_val( &[ "token" ], default );       
        expand_path
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
        let default = "~/.local/share/ai/%profile%/models/%provider%.txt".to_string();
        let path = self.get_config_val( &[ "model" ], default );
        
        expand_path
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

        if let Err(e) = ensure_directory( &file_path )
        {
            self.application.get_log()
            .error( "Failed to ensure model directory" )
            .prm( "error", &e );
            return self;
        }

        if let Err(e) = std::fs::write( &file_path, id )
        {
            self.application.get_log()
            .error( "Failed to switch model" )
            .prm( "path", &file_path )
            .prm( "error", &e.to_string() );
        } 
        else
        {
            self.application.get_log()
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
            "~/.config/ai/%profile%/provider.txt".to_string()
        );
        
        expand_path( &path.replace( "%profile%", &self.get_profile() ))
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

        if let Err(e) = ensure_directory(&file_path)
        {
            self.application.get_log()
            .error( "Failed to ensure provider directory" )
            .prm( "error", &e );

            return self;
        }

        if let Err( e ) = std::fs::write( &file_path, new_provider )
        {
            self.application.get_log()
            .error("Failed to switch provider")
            .prm("path", &file_path)
            .prm("error", &e.to_string());
        } 
        else
        {
            self.application.get_log()
            .trace("Provider switched")
            .prm("provider", new_provider);
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
        expand_path( "~/.local/share/ai/profile" )
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
            /* Set state for application */
            self.state.set_state
            (
                "PROFILE_WRITE_ERROR",
                serde_json::json!
                (
                    {
                        "path": path,
                        "error": e.to_string()
                    }
                )
            );
            /* Write in to log */
            self.application.get_log()
            .error("Failed to write profile")
            .prm("path", &path)
            .prm("error", &e.to_string());
        } 
        else
        {
            self.application.get_log()
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
            "~/.local/share/ai/%profile%/chat.txt".to_string()
        );
        
        expand_path
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
        
        if let Err(e) = ensure_directory( &file_path )
        {
            self.application.get_log()
            .error("Failed to ensure chat directory")
            .prm("error", &e);
            return self;
        }
        
        if let Err(e) = std::fs::write(&file_path, new_id)
        {
            self.application.get_log()
            .error("Failed to switch chat")
            .prm("path", &file_path)
            .prm("error", &e.to_string());
        } 
        else
        {
            self.application.get_log()
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
        dest_type: &str
    )
    {
        let command = self.get_config_val
        (
            &["destination", dest_type], 
            String::new()
        );
        self.run_command( data, &command, false );
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
                            self.application.get_log()
                            .info( "Command executed successfully" )
                            .prm( "command", command )
                            .prm( "data_bytes", data_len )
                            .prm( "exit_code", exit_status.code().unwrap_or( -1 ));
                        }
                        Err( e ) =>
                        {
                            self.application.get_log()
                            .warning("Failed to wait for command")
                            .prm("command", command)
                            .prm("error", &e.to_string());
                        }
                    }
                }
                else 
                {
                    self.application.get_log()
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
                .application.get_log()
                .error( "Failed to execute command" )
                .prm( "command", command )
                .prm( "data_bytes", data.len() )
                .prm( "error", &e.to_string() );
                println!( "{}", data );
            }
        }
    }



    /*
        Processing chat responce
    */
    pub fn handle_chat_response
    (
        &mut self,
        response: &ChatResponse
    )
    {
        /* Replace %pool% placeholder in all fields */
        let pool_path = self.get_pool_path();
        let message = response.message.replace( "%pool%", &pool_path );
        let command = response.command.replace( "%pool%", &pool_path );
        let pool = response.pool.replace( "%pool%", &pool_path );
        let memory = response.memory.replace( "%pool%", &pool_path );
        let clipboard = response.clipboard.replace( "%pool%", &pool_path );

        /* Write to history if has content */
        if !message.is_empty() || !command.is_empty() 
        {
            self.write_history
            (
                "@AI",
                &format!
                (
                    "{}\n{}\n\n",
                    message,
                    command
                )
            );
        }

        /* Output message via destination */
        if !message.is_empty()
        {
            self.run_destination( &message, "message" );
        }

        /* Put information to clipboard */
        if !clipboard.is_empty()
        {
            self.run_destination( &clipboard, "clipboard" );
        }

        /* Execute command via destination */
        if !command.is_empty()
        {
            /* Check if command execution is disabled */
            let no_command = self.application.config
            .as_ref()
            .and_then(|cfg| cfg["no-command"].as_bool())
            .unwrap_or( false );

            if no_command
            {
                self.application.get_log()
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
                let clean_command = command.replace('\n', " ").replace('\r', "");
                self.run_destination(&clean_command, "command" );
            }
        }

        /* Write pool via destination */
        if !pool.is_empty()
        {
            self.run_destination( &pool, "pool" );
        }

        /* Save memory */
        if !memory.is_empty()
        {
            self.write_memory(&memory);
        }

        /* Log token usage */
        self.application.get_log()
        .trace("Token usage")
        .prm("prompt_tokens", response.prompt_tokens)
        .prm("answer_tokens", response.answer_tokens);
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
                        self.application.get_log()
                        .error( "TIOCSTI ioctl failed" )
                        .prm( "byte", &byte.to_string())
                        .prm( "error", &std::io::Error::last_os_error().to_string());

                        break;
                    }
                }
                
                self.application.get_log()
                .info( "Command injected via TIOCSTI" )
                .prm( "tty", &tty_device )
                .prm( "length", cmd.len() );
            }
            Err(e) => 
            {
                self.application.get_log()
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
        Default: ~/.local/share/ai/%profile%/memory/%chat%.txt
    */
    fn get_memory_file( &self )
    -> String
    {
        expand_path
        (
            &self.get_config_val
            (
                &[ "memory" ], 
                "~/.local/share/ai/%profile%/memory/%chat%.txt".to_string()
            )
            .replace( "%profile%", &self.get_profile() )
            .replace( "%provider%", &self.get_provider() )
            .replace( "%model%", &self.get_model_safe() )
            .replace( "%chat%", &self.get_chat() )
        )
    }



    /*
        Return current memory file size in bytes
    */
    fn get_memory_size_byte( &self)
    -> usize 
    {
        let path = self.get_memory_file();
        std::fs::metadata( &path )
        .map(|m| m.len() as usize)
        .unwrap_or(0)
    }



    /*
        Clear memory file for current chat
    */
    fn clear_memory( &mut self )
    -> &mut Self
    {
        let path = self.get_memory_file();
        
        match std::fs::write(&path, "")
        {
            Ok( _ ) =>
            {
                self.application.get_log()
                .info("Memory cleared")
                .prm("path", &path);
            }
            Err( e ) =>
            {
                self.application.get_log()
                .warning( "Failed to clear memory" )
                .prm( "path", &path)
                .prm( "error", &e.to_string());
            }
        }
        
        self
    }



    /*
        Write memory
    */
    fn write_memory
    (
        &mut self, 
        text: &str
    )
    {
        let memory_path = self.get_memory_file();

        /* Create parent directory */
        if let Err( e ) = ensure_directory( &memory_path )
        {
            self.application.get_log()
            .error( "Failed to ensure memory directory" )
            .prm( "error", &e );
            return;
        }

        use std::fs::OpenOptions;
        use std::io::Write;

        let line = format!
        (
            "@FACT {}\n{}\n\n", 
            &Moment::create().now().format( "%Y-%m-%d %H:%M:%S" ),
            text
        );

        if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&memory_path)
        {
            let _ = file.write_all(line.as_bytes());
        }
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
        self.application.get_log()
        .dump("Prompt to LLM", prompt)
        .prm("provider", provider)
        .prm("model", model)
        .prm("api", api_url)
        .prm("type", prompt_type);
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
        self.application.get_log()
        .dump("Response from LLM", response)
        .prm("provider", provider)
        .prm("model", model)
        .prm("api", api_url)
        .prm("type", prompt_type);
    }
}

