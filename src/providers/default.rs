use regex::Regex;
use reqwest::blocking::Response;

use core::Color;

use crate::Ai;
use crate::ai::response:: ChatResponse;
use super::api::{get_api_url, get_token};
use super::Provider;



/*
    Default Provider for OpenAI-compatible APIs.
    Supports: github, openai, deepseek, groq, together.
*/
pub struct OpenAICompatibleProvider <'a>
{
    /* Shared reference to AI instance (config, token, logs) */
    ai: &'a mut Ai,
    /* Provider name (github, openai, deepseek, groq, together) */
    name: String,
}



impl<'a> OpenAICompatibleProvider<'a>
{
    /*
        Create new default provider instance.
    */
    pub fn new
    (
        /* Provider name */
        name: &str,
        /* AI application instance (shared ownership) */
        ai: &'a mut Ai
    )
    -> Self
    {
        Self
        {
            ai: ai,
            name: name.to_string(),
        }
    }



    fn create_client( &self )
    -> reqwest::blocking::Client
    {
        let mut builder = reqwest::blocking::Client::builder();       
        let proxy_url = self.ai.read_proxy();
        if !proxy_url.is_empty() 
        {
            if let Ok(proxy) = reqwest::Proxy::all(&proxy_url) 
            {
                builder = builder.proxy(proxy);
            }
        }
        
        builder.build().unwrap()
    }



    fn parse_openai_response
    (
        &mut self,
        raw: String
    )
    ->
    (
        /* Error text */
        String,
        /* think text */
        String,
        /* content text */
        String,
        /* prompt_tokens count */
        u64,
        /* answer_tokens */
        u64,
        /* sucess true or false */
        bool
    )
    {
        /* Result variables */
        let mut error_msg = String::new();
        let think;
        let content;
        let mut prompt_tokens = 0;
        let mut completion_tokens = 0;
        let success;

        let raw = raw.trim();

        /* Split think and raw */
        let re = Regex::new( r"(?s)<think>(.*?)</think>" ).unwrap();
        think = re.captures( raw )
            .and_then( |cap| cap.get( 1 ).map( |m| m.as_str().to_string() ) )
            .unwrap_or_default();
        let clear = re.replace_all( raw, "" ).to_string();

        /* Get json from raw answer */
        match serde_json::from_str::<serde_json::Value>( &clear )
        {
            Err( e ) =>
            {
                self.ai.application.get_log()
                    .error( "Failed to parse response" )
                    .prm( "error", &e.to_string() )
                    .prm( "content", &clear );

                error_msg = e.to_string();
                content = clear.to_string();
                success = false;
            }

            Ok( json ) =>
            {
                if let Some(error) = json.get("error") {
                    error_msg = error[ "message" ]
                        .as_str()
                        .unwrap_or("Unknown API error")
                        .to_string();
                    content = error_msg.clone();
                    success = false;
                } 
                else
                {
                    content = json
                        .get("choices")
                        .and_then(|c| c.get(0))
                        .and_then(|c| c.get("message"))
                        .and_then(|m| m.get("content"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .trim()
                        .trim_start_matches("```json")
                        .trim_start_matches("```")
                        .trim_end_matches("```")
                        .trim()
                        .to_string();

                    prompt_tokens = json
                        .get("usage")
                        .and_then(|u| u.get("prompt_tokens"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);

                    completion_tokens = json
                        .get("usage")
                        .and_then(|u| u.get("completion_tokens"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);

                    success = true;
                }
            }
        }

        ( 
            error_msg, 
            think, 
            content, 
            prompt_tokens, 
            completion_tokens, 
            success
        )
    }



    /*
       Dump response headers.
    */
    fn dump_headers( &mut self, resp: &Response )
    {
        self.ai.application.get_log().begin( "Response headers" );

        for( name, value ) in resp.headers().iter()
        {
            self.ai.application.get_log().trace( "" ).prm
            (
                name.as_str(),
                value.to_str().unwrap_or( "N/A" )
            );
        }

        self.ai.application.get_log().end( "" );
    }
}



impl<'a> Provider for OpenAICompatibleProvider<'a>
{
    /*
        Return provider name identifier.
    */
    fn get_name( &self ) 
    -> &str
    {
        &self.name
    }



    /*
        Send chat request and parse response.
    */
    fn chat( &mut self )
    {
        let user_prompt = self.ai.get_user_prompt();
        let prompt = self.ai.build_prompt( "chat", &user_prompt );

        /* Write user prompt to history */
        self.ai.write_history( self.ai.get_history_delimiter(), "" );
        self.ai.write_history( "@USER", &user_prompt );

        let api_url = get_api_url( self.ai, &self.name );
        let token = get_token( self.ai );
        let model = self.ai.get_model();
        let client = self.create_client();

        /* Trigger before request event */
        self.ai.on_before_request( &prompt, &self.name, &model, &api_url, "chat" );
    
        /* Prepare request */
        let payload = serde_json::json!
        (
            {
                "messages": [{ "role": "user", "content": prompt }],
                "model": model
            }
        );

        /*
            Request
        */
        let response = client.post( &api_url )
        .bearer_auth( &token )
        .header( "Content-Type", "application/json" )
        .json( &payload )
        .send();

        /*
            Control result
        */        
        match response
        {
            Ok( resp ) =>
            {
                /* Dump headers */
                self.dump_headers( &resp );

                /* Get full answer */             
                let full_answer = resp.text().unwrap_or_default();

                /* Event */
                self.ai.on_after_response( &full_answer, &self.name, &model, &api_url, "chat");

                /* Get openai fields*/
                let
                (
                    error,
                    think, 
                    content, 
                    prompt_tokens, 
                    answer_tokens,
                    result
                ) = self.parse_openai_response( full_answer );

                let mut chat_response = ChatResponse
                {
                    think,
                    message: if error.is_empty() { content } else { format!("Error: {}", error) },
                    prompt_tokens,
                    answer_tokens,
                    clipboard: String::new(),
                    command: String::new(),
                    pool: String::new(),
                    memory: String::new()
                };
                
                if result
                {
                    /* Get ai tool json from content */
                    match serde_json::from_str::<serde_json::Value>( &chat_response.message )
                    {
                        Ok( ai_json ) =>
                        {
                            chat_response.command = ai_json[ "command" ]
                            .as_str()
                            .unwrap_or( "" )
                            .to_string();

                            chat_response.message = ai_json[ "message" ]
                            .as_str()
                            .unwrap_or( "" )
                            .to_string()
                            .replace("\\n", "\n");
                            
                            chat_response.pool = ai_json[ "pool" ]
                            .as_str()
                            .unwrap_or( "" )
                            .to_string()
                            .replace("\\n", "\n");
                            
                            chat_response.memory = ai_json[ "memory" ]
                            .as_str()
                            .unwrap_or( "" )
                            .to_string()
                            .replace( "\\n", "\n" );

                            chat_response.clipboard = ai_json[ "clipboard" ]
                            .as_str()
                            .unwrap_or( "" )
                            .to_string()
                            .replace("\\n", "\n");
                        }
                        Err( _ ) =>
                        {
                        }
                    }
                }
                self.ai.handle_chat_response( &chat_response );
            }
            Err( e ) =>
            {
                /* event */
                self.ai.on_after_response
                (
                    &e.to_string(), 
                    &self.name, 
                    &model, 
                    &api_url, 
                    "chat"
                );

                println!
                (
                    "{}{}\n{}{}",
                    Color::Red.to_str(),
                    "API error",
                    &e.to_string(),
                    Color::Default.to_str()
                );

                let provider_name = self.get_name().to_string();
                let proxy = self.ai.read_proxy();
                self.ai.application.get_log()
                .error( "API error" )
                .prm( "error", &e.to_string() )
                .prm( "provider", provider_name )
                .prm( "api", api_url )
                .prm( "proxy", proxy )
                ;
            }
        }      
    }



    /*
        Send summarization request and parse response.
    */
    fn summary
    (
        &mut self,
        percent: u64
    )
    {
        self.ai.application.get_log().begin( "summary" );

        let history = self.ai.get_history();

        if history.is_empty()
        {
            self.ai.application.get_log().info( "No history to summarize" );
            return;
        }

        let blocks: Vec<&str> = history.split(self.ai.get_history_delimiter()).collect();
        if blocks.len() < 2
        {
            self.ai.application.get_log().info( "Not enough blocks to summarize" );
            return;
        }

        /* Calculate split point based on percent (0-100) */
        let percent = percent.clamp( 0, 100 );
        let split_point = (blocks.len() as f64 * (percent as f64 / 100.0)).round() as usize;
        let split_point = split_point.max(1).min(blocks.len() - 1);

        let old_history = blocks[..split_point].join(self.ai.get_history_delimiter());
        let recent_history = blocks[split_point..].join(self.ai.get_history_delimiter());

        let old_blocks = split_point;
        let kept_blocks = blocks.len() - split_point;

        /* Build summary prompt */
        let prompt = self.ai.build_prompt( "summary", &old_history );

        /* Send request */
        let api_url = get_api_url(self.ai, &self.name);
        let token = get_token(self.ai);
        let model = self.ai.get_model();
        let client = self.create_client();

        /* Trigger before request event */
        self.ai.on_before_request
        (
            &prompt, 
            &self.name, 
            &model, 
            &api_url, 
            "summary"
        );

        let payload = serde_json::json!
        ({
            "messages": [{ "role": "user", "content": prompt }],
            "model": model
        });

        let response = client.post(&api_url)
        .bearer_auth(&token)
        .json(&payload)
        .send();

        match response
        {
            Ok(resp) =>
            {
                /* Get raw answer */
                let raw_answer = resp.text().unwrap_or_default();

                /* Event */
                self.ai.on_after_response
                (
                    &raw_answer,
                    &self.name,
                    &model,
                    &api_url,
                    "summary"
                );

                let
                (
                    error,
                    think, 
                    summary, 
                    prompt_tokens, 
                    answer_tokens, 
                    success
                ) = self.parse_openai_response(raw_answer);

                self.ai.handle_summary_response
                (
                    &summary,
                    &error,
                    &think,
                    &recent_history,
                    prompt_tokens,
                    answer_tokens,
                    success,
                    old_blocks,
                    kept_blocks
                );
            }
            Err(e) =>
            {
                self.ai.on_after_response
                (
                    &e.to_string(), 
                    &self.name, 
                    &model, 
                    &api_url, 
                    "summary"
                );
                self.ai.application.get_log()
                    .error( "Summary request failed" )
                    .prm( "error", &e.to_string() );
            }
        }

        self.ai.application.get_log().end( "" );
    }
}
