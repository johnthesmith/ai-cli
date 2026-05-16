use core::expand_path;
use core::Moment;
use serde_json;

/* sudo apt install libxdo-dev */
use enigo::{Enigo, Keyboard, Settings};

/*
    Main AI module
*/
use core::Application;


/*
    Ai applicatoin
*/
pub struct Ai 
{
    pub application: Application,
    pub kbd_response: String
}



/*
    Ai implementation
*/
impl Ai 
{

    pub fn create() -> Self 
    {
        Self 
        {
            application: Application::create(),
            kbd_response: String::new(),
        }
    }



    pub fn run
    (
        &mut self
    ) 
    -> &mut Self 
    {
        /* Get default config path */
        let path = std::env::var("HOME").unwrap() + "/.config/ai/default/config.yaml";
        /* Read config */
        self.application.read_config( &path ).read_cli().dump_config();

        /* First log message */
        self.application.get_log().begin( "Ai started" );

      

        /* Check config */
        if self.application.state.is_ok()
        {
            /* Let chat id */
            let chat_id = self.get_chat_id();
            let new_chat_id = self.application.config
                .as_ref()
                .and_then(|cfg| cfg[ "chat" ].as_str())
                .unwrap_or( &chat_id )
                .to_string();
                
            if chat_id != new_chat_id
            {
                self.switch_chat_id( &new_chat_id );
            }

            /* Check clear-history flag */
            let clear_history = self.application.config
                .as_ref()
                .and_then(|cfg| cfg["clear"].as_bool())
                .unwrap_or(false);

            /* Help mode */
            let help_mode = self.application.config
                .as_ref()
                .and_then(|cfg| cfg["help"].as_bool())
                .unwrap_or( false );
            
            /* History request */
            let history = self.application.config
                .as_ref()
                .and_then(|cfg| cfg["history"].as_bool())
                .unwrap_or( false );
            
            if help_mode
            {
                self.help();
            }
            else
            {
                if history
                {
                    self.show_history();
                }
                else
                {
                    if clear_history
                    {
                        self.clear_history();
                    }
                    else
                    {
                        self.write_history("user", &self.get_user_prompt());
                        self.request();
                    }
                }
            }
        }

        self.application.get_log().end( "" ).eol();

        /* Final output leyboard */
        if !self.kbd_response.is_empty() 
        {
            std::thread::sleep(std::time::Duration::from_millis(100));
            match Enigo::new(&Settings::default()) 
            {
                Ok( mut enigo ) => 
                {
                    let _ = enigo.text(&self.kbd_response);
                    // let _ = enigo.text("\n");
                }
                Err(e) => 
                {
                    self.application.get_log()
                        .error("Failed to init enigo")
                        .prm("error", &e.to_string())
                        .eol();
                }
            }
            println!();
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
                let full_path = expand_path(&path);
                match std::fs::read_to_string(&full_path) {
                    Ok( content ) => content,
                    Err(_) => 
                    {
                        self.application.get_log()
                        .warning( "Cannot read prompt file, using default" )
                        .prm("path", full_path)
                        .eol();

                        default_prompt
                    }
                }
            }
            None => default_prompt
        }
    }



    /*
        Return user prompt from first argument
    */
    fn get_user_prompt(&self) -> String 
    {
        /* Reading from config */
        if let Some(prompt) = self.application.config
            .as_ref()
            .and_then(|cfg| cfg["_0"].as_str())
        {
            return prompt.to_string();
        }
        
        /* else from stdin */
        use std::io::Read;
        
        let mut buffer = String::new();
        let mut stdin = std::io::stdin();
        
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



    fn prepare_prompt(&mut self) -> String 
    {
        let system_prompt = self.read_prompt();
        let history = self.get_history();
        let user_prompt = self.get_user_prompt();
        let chat = self.get_chat_id();
        
        system_prompt
            .replace("%chat%", &chat)
            .replace("%history%", &history)
            .replace("%user-prompt%", &user_prompt)
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
            .and_then(|cfg| cfg["application"]["ai"]["provider"].as_str())
            .unwrap_or("unknown")
            .to_string()
    }



    /*
        Request and return result
    */
    fn request( &mut self )
    -> &mut Self
    {
        let provider_type = self.get_provider();

        self
        .application.get_log()
        .begin( "Request" )
        .prm( "provider-type", &provider_type );
        
        match provider_type.as_str() 
        {
            "github" => 
            {
                let response = self.request_github();
                let (in_cmd, out_msg, prompt_tokens, answer_tokens ) = self.answer_github( response );

                /* Write answer in to history */
                self.write_history("ai", &format!("{}\n{}", out_msg, in_cmd));

                /* Output result in to stdout */
                println!("{}", out_msg);

                /* Set keyboard input with chek enter */
                self.kbd_response = in_cmd
                    .replace( ['\n', '\r'], " ")
                    .trim()
                    .to_string();

                self.application.get_log()
                    .trace( "Success answer" )
                    .prm("prompt-tokens", prompt_tokens)
                    .prm("answer-tokens", answer_tokens)
                    .eol();
            }
            _ =>
            {
                self.application.get_log()
                    .warning( "Unknown provider" )
                    .prm("type", &provider_type)
                    .eol();
            }
        }

        self.application.get_log().end( "" );

        self
    }



    /* 
        GitHub AI request
    */
    fn request_github( &mut self ) -> String 
    {
        let api = self.application.config
            .as_ref()
            .and_then(|cfg| cfg["application"]["ai"]["params"]["api"].as_str())
            .unwrap_or("")
            .to_string();
        
        let token_path = self.application.config
            .as_ref()
            .and_then(|cfg| cfg["application"]["ai"]["params"]["token"].as_str())
            .unwrap_or("");
        
        let token = if let Ok(content) = std::fs::read_to_string(expand_path(token_path)) {
            content.trim().to_string()
        }
        else 
        {
            String::new()
        };
        
        let model = self.application.config
            .as_ref()
            .and_then(|cfg| cfg["application"]["ai"]["params"]["model"].as_str())
            .unwrap_or("openai/gpt-4o-mini")
            .to_string();
        
        let prompt = self.prepare_prompt();
        
        let response = ureq::post(&api)
            .set("Authorization", &format!("Bearer {}", token))
            .set("Content-Type", "application/json")
            .send_json(serde_json::json!({
                "messages": [{"role": "user", "content": prompt}],
                "model": model
            }));
        
        match response 
        {
            Ok(resp) => 
            {
                resp.into_string().unwrap_or_default()
            }
            Err(e) => 
            {
                self.application.get_log()
                    .error("GitHub API error")
                    .prm("error", &e.to_string())
                    .eol();
                String::new()
            }
        }
    }




    /*
        Github AI responce
    */
    fn answer_github
    (
        &mut self,
        /* String with responce after request_github */
        response: String
    )
    ->
    (
        /* In for tty input */
        String,
        /* Out for std out */
        String,
        /* Prompt tokens count */
        u64,
        /* Completion tokens count */
        u64
    )
    {
        let json: serde_json::Value = serde_json::from_str(&response).unwrap_or(serde_json::json!({}));
        
        let prompt_tokens = json["usage"]["prompt_tokens"].as_u64().unwrap_or(0);
        let completion_tokens = json["usage"]["completion_tokens"].as_u64().unwrap_or(0);
        
        let content = json["choices"][0]["message"]["content"].as_str().unwrap_or("");
        let content = content
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```");

        // Пробуем распарсить как JSON
        match serde_json::from_str::<serde_json::Value>(content)
        {
            Ok(ai_json) =>
            {
                let out_msg = ai_json["out"].as_str().unwrap_or("").to_string();
                let in_cmd = ai_json["in"].as_str().unwrap_or("").to_string();

                if out_msg.is_empty() && in_cmd.is_empty()
                {
                    // Не структурирован, но может быть чистый JSON без полей in/out
                    (String::new(), content.to_string(), prompt_tokens, completion_tokens)
                }
                else
                {
                    (in_cmd, out_msg, prompt_tokens, completion_tokens)
                }
            }
            Err(_) =>
            {
                // Не структурирован — весь content в out
                (String::new(), content.to_string(), prompt_tokens, completion_tokens)
            }
        }
    }


    /**************************************************************************
        History
    */


    fn get_history_file_path(&mut self) -> Option<String>
    {
        let history_dir = self.application.config
            .as_ref()
            .and_then(|cfg| cfg["application"]["ai"]["history"].as_str())
            .map(|s| expand_path(s));
        
        let chat_id = self.get_chat_id();
        
        history_dir.map(|dir| format!("{}{}.txt", dir, chat_id))
    }


    fn get_history(&mut self) -> String 
    {
        let history_path = match self.get_history_file_path() {
            Some(path) => path,
            None => return String::new(),
        };
        
        match std::fs::read_to_string(&history_path) {
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

        if let Some(path) = history_path {
            use std::fs::OpenOptions;
            use std::io::Write;

            let mut now = Moment::create();
            now.now();
            let micros = now.get();

            let line = format!("@LINE\n{}\n{}\n{}\n\n", role, micros, text);

            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path) {
                let _ = file.write_all(line.as_bytes());
            }
        }
    }



    /*
        Clear history file
    */
    fn clear_history( &mut self )
    -> &mut Self
    {
        let history_path = self.get_history_file_path();
        
        match history_path
        {
            Some(path) =>
            {
                match std::fs::write(&path, "")
                {
                    Ok(_) =>
                    {
                        self.application.get_log()
                            .info("History cleared")
                            .prm("path", &path)
                            .eol();
                    }
                    Err(e) =>
                    {
                        self.application.get_log()
                            .warning("Failed to clear history")
                            .prm("path", &path)
                            .prm("error", &e.to_string())
                            .eol();
                    }
                }
            }
            None =>
            {
                self.application.get_log()
                    .warning("No history file configured")
                    .eol();
            }
        }
        
        self
    }



    fn show_history(&mut self) -> &mut Self 
    {
        let history = self.get_history();
        if history.is_empty() {
            println!("No history");
        } else {
            println!("{}", history);
        }
        self
    }

    /*******************************************************************8******
        Chat
    */

    fn get_chat_file_path(&self) -> Option<String> 
    {
        self.application.config
            .as_ref()
            .and_then(|cfg| cfg["application"]["ai"]["chat-file"].as_str())
            .map(|s| expand_path(s))
    }

    fn get_chat_id(&self) -> String {
        if let Some(path) = self.get_chat_file_path() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let id = content.trim().to_string();
                if !id.is_empty() {
                    return id;
                }
            }
        }
        "default".to_string()
    }



    fn switch_chat_id(&mut self, new_id: &str) -> &mut Self {
        if let Some(path) = self.get_chat_file_path() {
            if let Err(e) = std::fs::write(&path, new_id) {
                self.application.get_log()
                    .error("Failed to switch chat")
                    .prm("path", &path)
                    .prm("error", &e.to_string())
                    .eol();
            } else {
                self.application.get_log()
                    .info("Chat switched")
                    .prm("id", new_id)
                    .eol();
            }
        }
        self
    }



    fn help(&mut self) 
    -> &mut Self
    {
        println!("AI CLI Utility");
        println!("");
        println!("Usage:");
        println!("  ai <question>               Ask a question");
        println!("  echo <text> | ai            Read from stdin");
        println!("  ai --help                   Show this help");
        println!("");
        println!("Options:");
        println!("  --chat <id>                 Switch to chat <id>, default id is default");
        println!("  --clear                     Clear current chat history");
        println!("  --history                   Show history for current chat");
        println!("  --help                      Show this help");
        self
    }
}


