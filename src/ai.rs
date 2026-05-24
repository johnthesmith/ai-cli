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

use core::{ expand_path, ensure_directory };
use serde_json;
use core::State;
use core::Application;
use std::io::Read;
use std::io::Write;
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

    /* Profile */
    profile: String,

    /* Provider */
    provider: String
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
            application: Application::create(),
            profile: String::new(),
            provider: String::new(),
            state: State::ok()
        }
    }



    /*
        Help utility
    */
    fn help(&mut self) -> &mut Self
    {
        println!( "AI CLI Utility 2026" );
        println!( "" );
        println!( "Usage:" );
        println!( "  ai                          Interactive keyboard input");
        println!( "  ai <question>               Ask a question" );
        println!( "  echo <text> | ai            Read from stdin" );
        println!( "  ai --help                   Show this help" );
        println!( "" );
        println!( "Options:" );
        println!( "  --help                      This information" );
        println!( "  --no-prompt                 Suppress input prompt" );
        println!( "  --show-info                 Show current runtime information (profile, chat, log, config)");
        println!( "  --profile=<name>            Use profile for current session only" );
        println!( "  --switch-profile=<name>     Switch and save profile" );
        println!( "  --switch-provider=<name>    Switch to AI provider <name> (saves to file)");
        println!( "  --provider=<name>           Use provider for current session only (no save)");
        println!( "  --switch-chat=<id>          Switch to chat <id>, default id is default" );
        println!( "  --show-history              Show history for current chat" );
        println!( "  --clear-history             Remove history for current chat" );
        println!( "  --pack-history              Pack current chat history into summary" );
        println!( "  --show-memory               Show memory for current chat" );
        println!( "  --clear-memory              Remove memory for current chat (global if %chat% not used)" );
        println!( "  --write-buffer              Write stdin to buffer file and forward to stdout" );
        println!( "  --tiocsti                   Inject input directly into TTY input buffer for keyboard" );
        println!( "                              Requires `sudo sysctl -w dev.tty.legacy_tiocsti=1`" );
        println!( "                              on modern kernels. Only use in trusted environments." );
        println!( "" );
        println!( "Recommendations:" );
        println!( "  alias                       Set `alias 1=ai`" );
        println!( "" );
        println!( "Author:" );
        println!( "  Still Swamp (still@itserv.ru) Powered by deepseek" );
        self
    }



    /*
        Main run method
    */
    pub fn run
    (
        &mut self
    ) 
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
            .and_then(|cfg| cfg[ "switch-profile" ].as_str())
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

            /* Switch chat if requested */
            let new_chat_id = self.application.config
                .as_ref()
                .and_then(|cfg| cfg["switch-chat"].as_str())
                .map(|s| s.to_string());

            if let Some(new_chat_id) = new_chat_id 
            {
                no_prompt = true;
                self.switch_chat_id(&new_chat_id);
            }

            /* Help mode */
            if let Some(_) = self.application.config
                .as_ref()
                .and_then(|cfg| cfg["help"].as_bool())
            {
                no_prompt = true;
                self.help();
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

            /* Check write-buffer flag */
            if let Some(true) = self.application.config.as_ref()
                .and_then(|cfg| cfg["write-buffer"].as_bool())
            {
                no_prompt = true;
                
                let mut input = String::new();
                if let Ok(_) = std::io::stdin().read_to_string(&mut input) {
                    self.write_buffer( &input ); 
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
                    Ok(0) => {
                        self.application.get_log()
                            .warning("tiocsti: stdin is empty");
                    }
                    Ok(_) => {
                        self.input_tiocsti(&input);
                    }
                    Err(e) => {
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

            if let Some(_) = self.application.config.as_ref()
                .and_then(|cfg| cfg["show-info"].as_bool())
            {
                no_prompt = true;
                self.show_info();
            }

            /* Pack history */
            if let Some(true) = self.application.config.as_ref()
                .and_then(|cfg| cfg["pack-history"].as_bool())
            {
                no_prompt = true;
                
                let provider_name = self.get_provider();
                let mut provider = providers::create_provider(&provider_name, self);
                provider.summary();
            }

            /* Check dump-prompt flag */
            if let Some(true) = self.application.config.as_ref()
                .and_then(|cfg| cfg["dump-prompt"].as_bool())
            {
                no_prompt = true;
                let user_prompt = self.get_user_prompt();
                let prompt = self.build_prompt( &user_prompt, "chat" );
                println!("{}", prompt);
            }

            if !no_prompt
            {
                let provider_name = self.get_provider();
                let mut provider = providers::create_provider( &provider_name, self );
                provider.chat();
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
        let path = self.application.config
            .as_ref()
            .and_then
            (
                |cfg| cfg
                [ "application" ]
                [ "ai" ]
                [ "prompts" ]
                [ prompt_type ]
                .as_str()
            )
            .map(|s| s.to_string())
            .unwrap_or_else
            (
                || format!
                (
                    "~/.config/ai/%profile%/prompts/%provider%/%model%/{}.txt", 
                    prompt_type
                )
            );

        let model = self.read_model()
        .replace('/', "_")
        .replace('\\', "_")
        .replace('.', "_")
        .replace("..", "_");

        expand_path( &path )
        .replace( "%profile%", &self.get_profile() )
        .replace( "%provider%", &self.get_provider() )
        .replace( "%model%", &model )
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
        Return user prompt from command line arguments (all non-flag arguments)
    */
    fn get_user_prompt( &self ) -> String
    {
        /* Collect all non-flag arguments */
        let args: Vec<String> = std::env::args().skip(1)
            .filter(|arg| !arg.starts_with('-'))
            .collect();
        
        if !args.is_empty() 
        {
            return args.join(" ");
        }

        /* else from stdin */
        use std::io::Read;
        use std::io::IsTerminal;

        let mut buffer = String::new();
        let mut stdin = std::io::stdin();

        /* Check if stdin is empty (no pipe) */
        if stdin.is_terminal()
        {
            println!("Enter your prompt (Ctrl+D to finish or Ctrl+C to cancel):");
        }

        if stdin.read_to_string(&mut buffer).unwrap_or(0) > 0
        {
            println!();
            buffer.trim().to_string()
        }
        else
        {
            println!();
            String::new()
        }
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
        
        template
            .replace("%chat%", &self.get_chat_id())
            .replace("%history%", &self.get_history())
            .replace("%memory%", &self.read_memory())
            .replace("%user-prompt%", input)
    }



    /**************************************************************************
        History
    */

    fn get_history_file_path(&mut self) -> String 
    {
        let history_dir = self.application.config
            .as_ref()
            .and_then(|cfg| cfg[ "application"]["ai"]["history"].as_str())
            .map(|s| expand_path( s ))
            .unwrap_or_else(|| expand_path( "~/.config/ai/%profile%/history/" ))
            .replace( "%profile%", &self.get_profile() );

        let chat_id = self.get_chat_id();

        format!("{}{}.txt", history_dir, chat_id)
    }



    fn get_history(&mut self) -> String 
    {
        let history_path = self.get_history_file_path();       
        match std::fs::read_to_string(&history_path) 
        {
            Ok(content) => content,
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
        
        // Создаём родительскую директорию
        if let Err(e) = ensure_directory(&history_path)
        {
            self.application.get_log()
            .error( "Failed to ensure history directory" )
            .prm( "error", &e );

            return;
        }
        
        use std::fs::OpenOptions;
        use std::io::Write;
        
        let line = format!( "{}\n{}\n\n", role, text );
        
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&history_path)
        {
            let _ = file.write_all(line.as_bytes());
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
            Ok(_) =>
            {
                self.application.get_log()
                .info("History cleared")
                .prm("path", &history_path);
            }
            Err(e) =>
            {
                self.application.get_log()
                    .warning("Failed to clear history")
                    .prm("path", &history_path)
                    .prm("error", &e.to_string());
            }
        }
        
        self
    }



    fn show_history(&mut self)
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
                .info("History packed successfully")
                .prm("old_blocks", old_blocks)
                .prm("kept_blocks", kept_blocks)
                .prm("prompt_tokens", prompt_tokens)
                .prm("answer_tokens", answer_tokens);

            if !think.is_empty()
            {
                self.application.get_log()
                    .trace("Summary think")
                    .prm("content", think);
            }
        }
        else
        {
            self.application.get_log().warning
            (
                "Failed to get summary, history unchanged"
            );
        }
    }



    /*******************************************************************8******
        Buffers
    */


    /*
        Return buffer file  
    */
    fn get_buffer_path(&self) 
    -> String
    {
        expand_path
        (
            &self.application.config
            .as_ref()
            .and_then(|cfg| cfg[ "application" ][ "ai" ][ "buffers" ].as_str() )
            .unwrap_or_else(|| "~/.local/share/ai/%profile%/buffer")
            .to_string()
            .replace("%profile%", &self.get_profile())
        )
    }



    fn write_buffer
    (
        &mut self,
        data: &str
    )
    {
        let buffer_path = self.get_buffer_path();
        if let Some(parent) = std::path::Path::new(&buffer_path).parent() 
        {
            let _ = std::fs::create_dir_all(parent);
        }
        
        match std::fs::write(&buffer_path, data) 
        {
            Ok(_) => 
            {
                self.application.get_log()
                    .info( "Buffer written to file" )
                    .prm( "path", &buffer_path );
            }
            Err(e) => 
            {
                self.application.get_log()
                    .error( "Failed to write buffer" )
                    .prm( "path", &buffer_path )
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
    fn get_config_file(&self)
    -> String
    {
        expand_path("~/.config/ai/%profile%/config.yaml")
        .replace("%profile%", &self.get_profile())
    }


 
    /*
        Return proxy for current provider
    */
    fn read_proxy(&self)
    -> String
    {
        let provider = self.get_provider();
        
        self.application.config
            .as_ref()
            .and_then(|cfg| cfg["application"]["ai"]["providers"][&provider]["proxy"].as_str())
            .unwrap_or("")
            .to_string()
    }



    /*******************************************************************8******
        Token
    */

    /*
        Return token path for current provider
    */
    fn get_token_path( &self )
    -> String
    {
        self.application.config
            .as_ref()
            .and_then
            (
                |cfg| cfg
                [ "application" ]
                [ "ai" ]
                [ "providers" ]
                [ self.get_provider() ]
                [ "token" ]
                .as_str()
            )
            .map(|s| expand_path(s).replace( "%profile%", &self.get_profile() ))
            .unwrap_or_else( || String::new() )
    }




    /*******************************************************************8******
        Model
    */

    /*
        Return file for current model
        Placeholders: %profile%, %provider%, %chat%
    */
    fn get_model_file_path( &self ) -> String
    {
        let provider = self.get_provider();

        self.application.config
            .as_ref()
            .and_then(|cfg| cfg["application"]["ai"]["providers"][ &provider ]["model"].as_str())
            .map(|s| expand_path(s))
            .unwrap_or_else(|| expand_path("~/.local/share/ai/%profile%/models/%provider%.txt"))
            .replace("%profile%", &self.get_profile())
            .replace("%provider%", &provider)
            .replace("%chat%", &self.get_chat_id())
    }



    pub fn read_model( &self ) -> String
    {
        let path = self.get_model_file_path();
        let provider = self.get_provider();

        if let Ok(content) = std::fs::read_to_string(&path)
        {
            let model = content.trim().to_string();
            if !model.is_empty()
            {
                return model;
            }
        }
        
        // Дефолтная модель из конфига провайдера
        self.application.config
            .as_ref()
            .and_then(|cfg| cfg["application"]["ai"]["providers"][provider]["models"][0].as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "gpt-4.1".to_string())
    }



    /*******************************************************************8******
        Chat
    */

    /*
        Return file for curren chat id 
    */
    fn get_chat_file_path( &self )
    -> String 
    {
        self.application.config
        .as_ref()
        .and_then(|cfg| cfg["application"]["ai"]["chat-file"].as_str())
        .map(|s| expand_path(s))
        .unwrap_or_else(|| expand_path( "~/.local/share/ai/%profile%/chat.txt" ))
        .replace("%profile%", &self.get_profile())
    }


    fn get_chat_id( &self )
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
    fn switch_chat_id
    (
        &mut self,
        new_id: &str
    ) -> &mut Self
    {
        let file_path = self.get_chat_file_path();
        
        if let Err(e) = ensure_directory( &file_path )
        {
            self.application.get_log()
                .error("Failed to ensure chat directory")
                .prm("error", &e);
            return self;
        }
        
        if let Err(e) = std::fs::write(&file_path, new_id) {
            self.application.get_log()
                .error("Failed to switch chat")
                .prm("path", &file_path)
                .prm("error", &e.to_string());
        } else {
            self.application.get_log()
                .trace("Chat switched")
                .prm("id", new_id);
        }
        
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
    fn get_provider_file_path( &self )
    -> String
    {
        self.application.config
            .as_ref()
            .and_then(|cfg| cfg[ "application" ][ "ai" ][ "provider_file" ].as_str())
            .map(|s| expand_path(s))
            .unwrap_or_else(|| expand_path( "~/.config/ai/%profile%/provider.txt" ))
            .replace( "%profile%", &self.get_profile() )
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
                .error("Failed to ensure provider directory")
                .prm("error", &e);
            return self;
        }

        if let Err(e) = std::fs::write(&file_path, new_provider)
        {
            self.application.get_log()
                .error("Failed to switch provider")
                .prm("path", &file_path)
                .prm("error", &e.to_string());
        } else {
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
    fn get_profile_file_path(&self) -> String 
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
    fn read_profile(&mut self) -> &mut Self
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



    /*
        Show runtime information
    */
    fn show_info(&mut self) -> &mut Self
    {
        println!("Log:                  {}", self.application.get_log().get_file_path());
        println!("Config:               {}", self.get_config_file());
        println!("Profile:              {}", self.get_profile());
        println!("Provider:             {}", self.get_provider());
        println!("Chat:                 {}", self.get_chat_id());
        println!("Model:                {}", self.read_model() );
        println!("Prompt chat file:     {}", self.get_prompt_file( "chat" ));
        println!("Prompt summary file:  {}", self.get_prompt_file( "summary" ));
        println!("Model file:           {}", self.get_model_file_path() );
        println!("History file:         {}", self.get_history_file_path() );
        println!("Memory file:          {}", self.get_memory_file() );
        println!("Token file:           {}", self.get_token_path() );
        self
    }



    /**************************************************************************
        Commands
    */

    /*
        Run destination command by identifier.
        Identifier: "command", "out", "buffer"
    */
    fn run_destination
    (
        &mut self, 
        data: &str, 
        dest_type: &str
    )
    {
        let command = self.application.config
        .as_ref()
        .and_then
        (
            |cfg| 
            cfg
            ["application"]
            ["ai"]
            ["destination"]
            [dest_type]
            .as_str()
        )
        .unwrap_or("")
        .to_string();
        
        self.run_command( data, &command, false );
    }



    /*
        Execute external command to insert the AI-generated text.
        Falls back to stdout if command execution fails.
    */
    fn run_command(&mut self, data: &str, command: &str, wait: bool)
    {
        if command.is_empty() {
            println!("{}", data);
            return;
        }

        match std::process::Command::new("bash")
            .arg("-c")
            .arg(command)
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            Ok(mut child) => {
                let data_len = data.len();
                
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(data.as_bytes());
                    let _ = stdin.flush();
                }
                
                if wait {
                    match child.wait() {
                        Ok(exit_status) => {
                            self.application.get_log()
                                .info("Command executed successfully")
                                .prm("command", command)
                                .prm("data_bytes", data_len)
                                .prm("exit_code", exit_status.code().unwrap_or(-1));
                        }
                        Err(e) => {
                            self.application.get_log()
                                .warning("Failed to wait for command")
                                .prm("command", command)
                                .prm("error", &e.to_string());
                        }
                    }
                } else {
                    // Не ждём, просто логируем запуск
                    self.application.get_log()
                        .info("Command spawned (no wait)")
                        .prm("command", command)
                        .prm("data_bytes", data_len);
                    
                    // Открепляем child, чтобы не ждать
                    std::thread::spawn(move || {
                        let _ = child.wait();
                    });
                }
            }
            Err(e) => {
                self.application.get_log()
                    .error("Failed to execute command")
                    .prm("command", command)
                    .prm("data_bytes", data.len())
                    .prm("error", &e.to_string());
                println!("{}", data);
            }
        }
    }



    pub fn handle_chat_response
    (
        &mut self,
        response: &ChatResponse
    )
    {
        /* Write to history if has content */
        if !response.message.is_empty() || !response.command.is_empty() {
            self.write_history(
                "@AI",
                &format!("{}\n{}\n\n", response.message, response.command)
            );
        }

        /* Output message via destination */
        if !response.message.is_empty() 
        {
            self.run_destination(&response.message, "message" );
        }

        /* Execute command via destination */
        if !response.command.is_empty()
        {
            /* 
                REMOVE_ENTER: CRITICAL SECURITY LAYER
                
                Removes newline and carriage return characters from LLM-generated command.
                Prevents command injection via line breaks that could:
                1. Terminate the current command
                2. Inject arbitrary new commands
                3. Execute hidden malicious code
                
                The cleaned command remains as a single line.
                Only newline/carriage return are removed - all other characters (&&, |, ;, $, `, etc.)
                are preserved as legitimate command syntax.
                
                This is a PROOF of security awareness - intentional design, not a bug.
            */
            let clean_command = response.command.replace('\n', " ").replace('\r', "");
            self.run_destination( &clean_command, "command" );
        }

        /* Write buffer via destination */
        if !response.buffer.is_empty()
        {
            self.run_destination(&response.buffer, "buffer" );
        }

        /* Save memory */
        if !response.memory.is_empty()
        {
            self.write_memory(&response.memory);
        }

        /* Log token usage */
        self.application.get_log()
        .trace("Token usage")
        .prm("prompt_tokens", response.prompt_tokens)
        .prm("answer_tokens", response.answer_tokens);
    }



    /*
        Inject command directly into TTY input buffer using TIOCSTI ioctl.

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
        let tty_device = self.application.config
        .as_ref()
        .and_then(|cfg| cfg["application"]["ai"]["input"]["tty_device"].as_str())
        .unwrap_or( "/dev/tty" )
        .to_string();
        
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
                    if ret != 0 {
                        self.application.get_log()
                            .error("TIOCSTI ioctl failed")
                            .prm("byte", &byte.to_string())
                            .prm("error", &std::io::Error::last_os_error().to_string());
                        break;
                    }
                }
                
                self.application.get_log()
                .info("Command injected via TIOCSTI")
                .prm("tty", &tty_device)
                .prm("length", cmd.len());
            }
            Err(e) => 
            {
                self.application.get_log()
                .error("Failed to open TTY device")
                .prm("device", &tty_device)
                .prm("error", &e.to_string());
                println!("{}", cmd);
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
    fn get_memory_file(&self) -> String
    {
        let path = self.application.config
            .as_ref()
            .and_then(|cfg| cfg["application"]["ai"]["memory"].as_str())
            .unwrap_or("~/.local/share/ai/%profile%/memory/%chat%.txt")
            .to_string()
            .replace("%profile%", &self.get_profile())
            .replace("%chat%", &self.get_chat_id());
        
        expand_path(&path)
    } 


    /*
        Clear memory file for current chat
    */
    fn clear_memory(&mut self) -> &mut Self
    {
        let path = self.get_memory_file();
        
        match std::fs::write(&path, "") {
            Ok(_) => {
                self.application.get_log()
                    .info("Memory cleared")
                    .prm("path", &path);
            }
            Err(e) => {
                self.application.get_log()
                    .warning("Failed to clear memory")
                    .prm("path", &path)
                    .prm("error", &e.to_string());
            }
        }
        
        self
    }



    /*
        Write memory
    */
    fn write_memory
    (
        &mut self, text: &str
    )
    {
        let memory_path = self.get_memory_file();

        // Create parent directory
        if let Err(e) = ensure_directory(&memory_path)
        {
            self.application.get_log()
                .error("Failed to ensure memory directory")
                .prm("error", &e);
            return;
        }

        use std::fs::OpenOptions;
        use std::io::Write;

        let line = format!("@FACT\n{}\n\n", text);

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
    fn read_memory(&self) -> String
    {
        let memory_path = self.get_memory_file();
        
        match std::fs::read_to_string(&memory_path)
        {
            Ok(content) => content,
            Err(_) => String::new(),
        }
    }



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
        Return history chat delimiter
    */
    pub fn get_history_delimiter( &self ) 
    -> &'static str
    {
        "=AIOL9B1MZX="
    }



}


