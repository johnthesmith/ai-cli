use crate::Ai;
use crate::ai::response::{ ChatResponse };
use super::api::{get_api_url, get_token};
use core::Color;
use super::Provider;
use regex::Regex;
use reqwest::blocking::Response;



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



    fn create_client(&self) -> reqwest::blocking::Client
    {
        let mut builder = reqwest::blocking::Client::builder();       
        let proxy_url = self.ai.read_proxy();
        if !proxy_url.is_empty() {
            if let Ok(proxy) = reqwest::Proxy::all(&proxy_url) {
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
    -> ( String, String, u64, u64, bool )
    {
        let raw = raw.trim();

        /* Split think and raw */
        let ( think, raw ) =
        {
            let re = Regex::new( r"(?s)<think>(.*?)</think>" ).unwrap();

            /* Extract think block content if exists */
            let think = re.captures( raw )
                .and_then( |cap| cap.get( 1 ).map( |m| m.as_str().to_string() ) )
                .unwrap_or_default();

            /* Remove think section from full text */
            let raw = re.replace_all( raw, "" ).to_string();

            /* Return separated parts */
            ( think, raw )
        };

        /* Get json from raw answer, it must contain OpenAI contract */
        match serde_json::from_str::<serde_json::Value>( &raw )
        {
            Err( e ) =>
            {
                self.ai.application.get_log()
                    .error( "Failed to parse response" )
                    .prm( "error", &e.to_string() )
                    .prm( "content", &raw );

                return ( think, raw, 0, 0, false );
            }

            Ok( json ) =>
            {
                /* Retrieve content */
                let content = json
                    .get( "choices" )
                    .and_then( |c| c.get( 0 ) )
                    .and_then( |c| c.get( "message" ) )
                    .and_then( |m| m.get( "content" ) )
                    .and_then( |v| v.as_str() )
                    .unwrap_or( "" )
                    .lines()
                    .map( |l| l.trim() )
                    .collect::<Vec<_>>()
                    .join( " " );

                /* Retrieve tokens */
                let prompt_tokens = json
                    .get( "usage" )
                    .and_then( |u| u.get( "prompt_tokens" ) )
                    .and_then( |v| v.as_u64() )
                    .unwrap_or( 0 );

                let completion_tokens = json
                    .get( "usage" )
                    .and_then( |u| u.get( "completion_tokens" ) )
                    .and_then( |v| v.as_u64() )
                    .unwrap_or( 0 );

                ( think, content, prompt_tokens, completion_tokens, true )
            }
        }
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
    fn get_name( &self ) -> &str
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
        let model = self.ai.read_model();
        let client = self.create_client();

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

                /* Dump full answer in to log */
                self.ai.application.get_log().dump( "Full answer", &full_answer );

                /* Get openai fields*/
                let
                ( 
                    think, 
                    content, 
                    prompt_tokens, 
                    answer_tokens,
                    result
                ) = self.parse_openai_response( full_answer );

                let mut chat_response = ChatResponse
                {
                    think,
                    message: content,
                    prompt_tokens,
                    answer_tokens,
                    command: String::new(),
                    buffer: String::new(),
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
                            
                            chat_response.buffer = ai_json[ "buffer" ]
                            .as_str()
                            .unwrap_or( "" )
                            .to_string()
                            .replace("\\n", "\n");
                            
                            chat_response.memory = ai_json[ "memory" ]
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
                println!
                (
                    "{}{} {}{}",
                    Color::Red.to_str(),
                    "API error",
                    &e.to_string(),
                    Color::Default.to_str()
                );
                let provider_name = self.get_name().to_string();
                self.ai.application.get_log()
                .error( "API error" )
                .prm( "error", &e.to_string() )
                .prm( "provider", provider_name );
            }
        }      
    }



    /*
        Send summarization request and parse response.
    */
    fn summary( &mut self )
    {
        let history = self.ai.get_history();

        if history.is_empty() {
            self.ai.application.get_log().info("No history to summarize");
            return;
        }

        let blocks: Vec<&str> = history.split(self.ai.get_history_delimiter()).collect();
        if blocks.len() < 2 {
            self.ai.application.get_log().info("Not enough blocks to summarize");
            return;
        }

        let mid = blocks.len() / 2;
        let old_history = blocks[..mid].join(self.ai.get_history_delimiter());
        let recent_history = blocks[mid..].join(self.ai.get_history_delimiter());

        /* Build summary prompt */
        let prompt = self.ai.build_prompt("summary", &old_history);

        /* Send request */
        let api_url = get_api_url(self.ai, &self.name);
        let token = get_token(self.ai);
        let model = self.ai.read_model();
        let client = self.create_client();

        let payload = serde_json::json!({
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
                let raw_answer = resp.text().unwrap_or_default();
                let (think, summary, prompt_tokens, answer_tokens, success) =
                self.parse_openai_response(raw_answer);
                self.ai.handle_summary_response
                (
                    &summary, 
                    &think, 
                    &recent_history, 
                    prompt_tokens, 
                    answer_tokens, 
                    success, 
                    mid, 
                    blocks.len() - mid
                );
            }
            Err(e) => {
                self.ai.application.get_log()
                    .error("Summary request failed")
                    .prm("error", &e.to_string());
            }
        }
    }
}
