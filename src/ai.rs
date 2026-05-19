/*
    Main AI module
*/

use core::{expand_path, ensure_directory };
use core::Color;
use serde_json;
use regex::Regex;
use core::State;
use core::Application;


/*
    Ai applicatoin
*/
pub struct Ai 
{
    /* Application structure */
    pub application: Application,

    /* Ai state structure */
    state: State,

    /* Keyboard buffer for final typing */
    kbd_response: String,

    /* Profile */
    profile: String
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
            kbd_response: String::new(),
            profile: String::new(),
            state: State::ok(),
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
        println!( "  --show-chat                 Show current chat id" );
        println!( "  --switch-chat=<id>          Switch to chat <id>, default id is default" );
        println!( "  --show-history              Show history for current chat" );
        println!( "  --clear-history             Remove history for current chat" );
        println!( "  --profile=<name>            Use profile for current session only" );
        println!( "  --switch-profile=<name>     Switch and save profile" );
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
        let path = 
        expand_path( "~/.config/ai/%profile%/config.yaml" )
        .replace( "%profile%", &self.get_profile() );

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

            /* Switch profile mode */
            if let Some(_) = self.application.config.as_ref()
                .and_then(|cfg| cfg["switch-profile"].as_str())
            {
                no_prompt = true;
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
            
            /* History request */
            if let Some(_) = self.application.config.as_ref()
                .and_then(|cfg| cfg["show-history"].as_bool())
            {
                no_prompt = true;
                self.show_history();
            }

            if let Some(_) = self.application.config.as_ref()
                .and_then(|cfg| cfg["show-info"].as_bool())
            {
                no_prompt = true;
                self.show_info();
            }
           
            if let Some(_) = self.application.config.as_ref()
                .and_then(|cfg| cfg["show-chat"].as_bool())
            {
                no_prompt = true;
                self.show_chat();
            }

            if !no_prompt
            {
                let system_prompt = self.read_prompt();
                let history = self.get_history();
                let user_prompt = self.get_user_prompt();
                let chat = self.get_chat_id();                          

                let prompt = system_prompt
                .replace( "%chat%", &chat )
                .replace( "%history%", &history )
                .replace( "%user-prompt%", &user_prompt );

                self.write_history( "USER", &user_prompt );
                self.request( prompt );
            }
        }

        self.application.get_log().end( "End of ai" ).eol();

        /* Final output leyboard */
        if !self.kbd_response.is_empty()
        {
            let cmd = self.kbd_response.clone();
            let config = &self.application.config;
            let mode = config
                .as_ref()
                .and_then(|cfg| cfg["application"]["ai"]["input"]["mode"].as_str())
                .unwrap_or("stdout");
            match mode 
            {
                "command" => self.input_command( &cmd ),
                "file" => self.input_file( &cmd ),
                "tiocsti" => self.input_tiocsti( &cmd ),
                _ => self.input_stdout( &cmd ),
            }
        }
        self
    }



    /**************************************************************************
        Prompt
    */

    fn read_prompt( &mut self ) -> String 
    {
        /* Set default prompt */
        let default_prompt = "%user-prompt%".to_string();
        
        let prompt_path = self.application.config
            .as_ref()
            .and_then(|cfg| cfg["application"]["ai"]["prompt"].as_str())
            .map(|s| s.to_string());
        
        match prompt_path 
        {
            Some(path) => 
            {
                let full_path = expand_path(&path)
                .replace("%profile%", &self.get_profile());
                
                match std::fs::read_to_string(&full_path) 
                {
                    Ok( content ) => content,
                    Err(_) => 
                    {
                        self.application.get_log()
                        .warning( "Cannot read prompt file, using default" )
                        .prm("path", full_path);

                        default_prompt
                    }
                }
            }
            None => default_prompt
        }
    }



    /*
        Return user prompt from command line arguments (all non-flag arguments)
    */
    fn get_user_prompt(&self) -> String
    {
        /* Collect all non-flag arguments */
        let args: Vec<String> = std::env::args().skip(1)
            .filter(|arg| !arg.starts_with('-'))
            .collect();
        
        if !args.is_empty() {
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



    /**************************************************************************
        Request
    */

    /*
        Return current provider
    */
    fn get_provider( &self )
    -> String
    {
        self.application.config
        .as_ref()
        .and_then(|cfg| cfg[ "application" ][ "ai" ][ "provider" ].as_str())
        .unwrap_or( "github" )
        .to_string()
    }



    /*
        Request and return result
    */
    fn request
    (
        &mut self,
        prompt: String
    )
    -> &mut Self
    {
        let provider_type = self.get_provider();

        self
        .application.get_log()
        .begin( "Request" )
        .prm( "provider-type", &provider_type );

        let mut in_cmd = String::new();
        let mut out_msg = String::new();
        let mut buffer = String::new();
        let mut prompt_tokens = 0;
        let mut answer_tokens = 0;

        let model = self.application.config
        .as_ref()
        .and_then(|cfg| cfg["application"]["ai"]["params"]["model"].as_str())
        .unwrap_or("openai/gpt-4o-mini")
        .to_string();

        /* Provider adapters */
        match provider_type.as_str() 
        {
            "github" => 
            {
                /* Request */
                let response = self.request_github( &model, &prompt );

                self.application.get_log().dump( "response", &response );
                
                /* Responce */
                (
                    in_cmd, 
                    out_msg, 
                    buffer, 
                    prompt_tokens, 
                    answer_tokens
                ) = self.answer_github( &model, &response );
            }
            _ =>
            {
                self.application.get_log()
                    .warning( "Unknown provider" )
                    .prm("type", &provider_type);
            }
        }

        /* Answer processing */
        if !in_cmd.is_empty() || !out_msg.is_empty() || !buffer.is_empty()
        {
            self.write_history( "AI", &format!("{}\n{}", out_msg, in_cmd));

            println!( "{}", out_msg );

            /* Write buffer to file */
            if !buffer.is_empty()
            {
                let path = self.get_buffer_path();
                if let Err(e) = std::fs::write(&path, &buffer)
                {
                    self.application.get_log()
                        .error( "Failed to write buffer" )
                        .prm( "error", &e.to_string() )
                        .eol();
                }
            }

            /* Store command to keyboard buffer */
            let buffer_path = self.get_buffer_path();
            self.kbd_response = in_cmd
                /* SECURE!!! REMOVE_ENTER from command */
                .replace(['\n', '\r'], " ")
                .replace("%buffer%", &buffer_path)
                .trim()
                .to_string();
            
            self.application.get_log()
                .trace("Success answer")
                .prm("prompt-tokens", prompt_tokens)
                .prm("answer-tokens", answer_tokens);
        }

        self.application.get_log().end( "" );

        self
    }



    /* 
        GitHub AI request
    */
    fn request_github
    (
        &mut self,
        /* Model id */
        model: &str,
        /* Prompt */
        prompt: &str
    ) -> String
    {
        let api = self.application.config
            .as_ref()
            .and_then(|cfg| cfg["application"]["ai"]["params"]["api"].as_str())
            .unwrap_or( "https://models.github.ai/inference/chat/completions" )
            .to_string();

        /* Retrive token */
        let token_path = self.application.config
            .as_ref()
            .and_then(|cfg| cfg["application"]["ai"]["params"]["token"].as_str())
            .map(|s| expand_path(s).replace("%profile%", &self.get_profile()))
            .unwrap_or_else(|| "".to_string());

        let token = if let Ok(content) = std::fs::read_to_string(token_path)
        {
            content.trim().to_string()
        }
        else
        {
            String::new()
        };

        /* Build reqwest client with proxy from config */
        let mut client_builder = reqwest::blocking::Client::builder();

        if let Some(proxy_url) = self.application.config
            .as_ref()
            .and_then(|cfg| cfg["application"]["ai"]["proxy"].as_str())
        {
            if let Ok(proxy) = reqwest::Proxy::all(proxy_url) {
                client_builder = client_builder.proxy(proxy);
            }
        }

        let client = client_builder.build().unwrap();


        let payload = serde_json::json!
        (
            {
                "messages": [{ "role": "user", "content": prompt }],
                "model": model
            }
        );

        let response = client.post(&api)
            .bearer_auth(&token)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send();

        if let Ok(resp) = &response
        {
            self.application.get_log().begin("GitHub response headers");

            for (name, value) in resp.headers().iter()
            {
                self.application.get_log()
                    .trace("")
                    .prm(name.as_str(), value.to_str().unwrap_or("N/A"));
            }

            self.application.get_log().end("");
        }

        match response
        {
            Ok(resp) =>
            {
                resp.text().unwrap_or_default()
            }
            Err(e) =>
            {
                println!
                (
                    "{}{} {}{}",
                    Color::Red.to_str(),
                    "GitHub API error",
                    &e.to_string(),
                    Color::Default.to_str()
                );
                self.application.get_log()
                    .error("GitHub API error")
                    .prm("error", &e.to_string());
                String::new()
            }
        }
    }



    /*
        Github AI response
    */
    fn answer_github
    (
        &mut self,
        /* Model */
        _model: &str,
        /* String with response after request_github */
        response: &str
    )
    ->
    (
        /* In for tty input */
        String,
        /* Out for stdout */
        String,
        /* Buffers content */
        String,
        /* Prompt tokens count */
        u64,
        /* Completion tokens count */
        u64
    )
    {
        let mut out_msg = response.to_string();
        let mut in_cmd = String::new();
        let mut buffer = String::new();
        let mut prompt_tokens = 0;
        let mut completion_tokens = 0;

        /* Remove think section if exists */
        let response_clean = Regex::new(r"(?s)<think>.*?</think>")
            .unwrap()
            .replace_all(&response, "")
            .to_string();

        /* Get json */
        match serde_json::from_str::<serde_json::Value>(&response_clean)
        {
            Err( e ) =>
            {
                out_msg = response.to_string();
                self.application.get_log()
                .error("Failed to parse GitHub response")
                .prm("error", &e.to_string());
            }
            Ok( json ) =>
            {
                /* Retrive content */
                let content = json["choices"][0]["message"]["content"]
                .as_str().unwrap_or( "" );

                /* Join all lines */
                let content = content
                    .lines()
                    .map(|l| l.trim())
                    .collect::<Vec<_>>()
                    .join(" ");

                /* Extract json source */
                let content = content
                    .trim()
                    .trim_start_matches("```json")
                    .trim_start_matches("```")
                    .trim_end_matches("```");
            
                /* Retrive tokens count */
                prompt_tokens = json[ "usage" ][ "prompt_tokens" ]
                .as_u64()
                .unwrap_or(0);

                completion_tokens = json[ "usage" ][ "completion_tokens" ]
                .as_u64()
                .unwrap_or(0);

                match serde_json::from_str::<serde_json::Value>( content )
                {
                    Ok( ai_json ) =>
                    {
                        out_msg = ai_json[ "out" ].as_str().unwrap_or(&out_msg).to_string().replace("\\n", "\n");
                        in_cmd = ai_json[ "in" ].as_str().unwrap_or("").to_string();
                        buffer = ai_json[ "buffer" ].as_str().unwrap_or("").to_string().replace("\\n", "\n");
                    }
                    Err( _ ) =>
                    {
                        /* If JSON parsing fails, return raw content as out_msg */
                        out_msg = if content.trim().is_empty() 
                        {
                            response.to_string()
                        }
                        else
                        {
                            content.to_string()
                        };
                    }           
                }
            }
        }

        (in_cmd, out_msg, buffer, prompt_tokens, completion_tokens )
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
        
        let line = format!( "@FROM_{}\n{}\n\n", role, text );
        
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



    fn show_chat( &mut self )
    -> &mut Self 
    {
        let chat_id = self.get_chat_id();
        println!( "Current chat: {}", chat_id );
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
        let model = self.application.config
        .as_ref()
        .and_then(|cfg| cfg["application"]["ai"]["params"]["model"].as_str())
        .unwrap_or("openai/gpt-4o-mini")
        .to_string();
    

        println!("Profile: {}", self.get_profile());
        println!("Chat: {}", self.get_chat_id());
        println!("Log: {}", self.application.get_log().get_file_path());
        println!("Config: ~/.config/ai/{}/config.yaml", self.get_profile());
        println!("Model: {}", model);
                
        self
    }



    /**************************************************************************
        Profile
    */
    fn input_stdout( &self, cmd: &str ) 
    {
        println!("{}", cmd);
    }



    /*
        Execute external command to insert the AI-generated text.
        Uses command template from config with %in% placeholder replaced by the actual command.
        Examples:
            xdotool type "%in%"
            echo -n "%in%" | xclip -selection clipboard
            tmux send-keys -t session "%in%"

        Falls back to stdout if command execution fails.
    */
    fn input_command
    (
        &mut self, 
        cmd: &str
    ) 
    {
        // Clone to release immutable borrow
        let command_template = self.application.config
            .as_ref()
            .and_then(|cfg| cfg["application"]["ai"]["input"]["command"].as_str())
            .unwrap_or("echo -n '%in%'")
            .to_string();
        
        let full_command = command_template.replace("%in%", cmd);
        
        // Log after releasing the borrow
        self.application.get_log()
            .info("Executing input command")
            .prm("template", &command_template)
            .prm("cmd", cmd);
        
        match std::process::Command::new("sh")
            .arg("-c")
            .arg(&full_command)
            .status()
        {
            Ok(status) if status.success() => 
            {
                self.application.get_log()
                    .info("Input command executed successfully");
            }
            Ok(status) => 
            {
                self.application.get_log()
                    .warning("Input command failed")
                    .prm("exit_code", status.code().unwrap_or(-1));
                // Fallback to stdout
                println!("{}", cmd);
            }
            Err(e) => 
            {
                self.application.get_log()
                    .error("Failed to execute input command")
                    .prm("error", &e.to_string());
                // Fallback to stdout
                println!("{}", cmd);
            }
        }
    }



    /*
        Write the AI-generated command to a file instead of typing it directly.
    */
    fn input_file
    (
        &mut self, 
        cmd: &str
    ) 
    {
        let file_path = self.application.config
        .as_ref()
        .and_then(|cfg| cfg["application"]["ai"]["input"]["file"].as_str())
        .map(|s| expand_path(s).replace("%profile%", &self.get_profile()))
        .unwrap_or_else(|| format!( "/tmp/ai-{}.sh", std::process::id()));
        
        if let Some(parent) = std::path::Path::new(&file_path).parent() 
        {
            let _ = std::fs::create_dir_all(parent);
        }
        
        match std::fs::write(&file_path, cmd) 
        {
            Ok(_) => 
            {
                self.application.get_log()
                    .info("Command saved to file")
                    .prm("path", &file_path);
            }
            Err(e) => {
                self.application.get_log()
                    .error("Failed to write command file")
                    .prm("path", &file_path)
                    .prm("error", &e.to_string());
                // Fallback to stdout
                println!("{}", cmd);
            }
        }
    }


    /*
        Inject command directly into TTY input buffer using TIOCSTI ioctl.

        This makes the command appear in the user's terminal prompt as if typed.
        Does NOT press Enter - user can edit before executing.

        # Security Warning
        Requires `sudo sysctl -w dev.tty.legacy_tiocsti=1` on modern kernels.
        Disabled by default due to security risks (CVE-2016-7545, CVE-2017-5223).
        Only use in trusted environments.

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
        .unwrap_or("/dev/tty")
        .to_string();  // Clone to release immutable borrow
        
        match std::fs::OpenOptions::new().write(true).open(&tty_device)
        {
            Ok(fd) =>
            {
                use std::os::unix::io::AsRawFd;
                let fd_raw = fd.as_raw_fd();
                
                for byte in cmd.bytes()
                {
                    let ret = unsafe {
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
                // Fallback to stdout
                println!("{}", cmd);
            }
        }
    }
}
