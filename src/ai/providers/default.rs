/*
    SPDX-License-Identifier: MIT
    SPDX-FileCopyrightText: 2026 Still Swamp
*/



use regex::Regex;
use reqwest::blocking::Response;

use crate::Ai;
use super::api::{ get_api_url, get_token };
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
        let mut builder =
        reqwest::blocking::Client::builder()
        .timeout( std::time::Duration::from_millis( self.ai.get_request_timeout_ms() ))
        .connect_timeout( std::time::Duration::from_millis( self.ai.get_connect_timeout_ms() ));

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



    /*
        Parse openai response from text and return information
    */
    fn parse_response
    (
        &mut self,
        raw: String
    )
    ->
    (
        /* Error text */
        String,
        /* Think text */
        String,
        /* Content text */
        String,
        /* Prompt tokens count */
        u64,
        /* Answer tokens count */
        u64,
        /* Sucess true or false */
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
                self.ai.app.get_log_mut()
                .error( "Failed to parse response" )
                .prm( "error", &e.to_string() )
                .prm( "content", &clear );

                error_msg = format!
                (
                    "Failed to parse response: {}",
                     e.to_string()
                );

                content = clear.to_string();
                success = false;
            }

            Ok( json ) =>
            {
                if let Some( error ) = json.get( "error" )
                {
                    error_msg = format!
                    (
                        "API error {}:",
                        error[ "message" ]
                        .as_str()
                        .unwrap_or( "Unknown API error" )
                        .to_string()
                    );

                    content = clear.to_string();
                    success = false;
                }
                else
                {
                    content = json[ "choices" ][ 0 ][ "message" ] ["content" ]
                    .as_str()
                    .unwrap_or( "" )
                    .trim()
                    .trim_start_matches( "```json" )
                    .trim_start_matches( "```" )
                    .trim_end_matches( "```" )
                    .trim()
                    .to_string();

                    prompt_tokens = json
                    [ "usage" ]
                    [ "prompt_tokens" ]
                    .as_u64()
                    .unwrap_or(0);

                    completion_tokens = json
                    [ "usage" ]
                    [ "completion_tokens" ]
                    .as_u64()
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
    fn dump_headers
    ( 
        &mut self, 
        resp: &Response
    )
    {
        self.ai.app.get_log_mut().begin( "Response headers" );

        for( name, value ) in resp.headers().iter()
        {
            self.ai.app.get_log_mut().trace( "" ).prm
            (
                name.as_str(),
                value.to_str().unwrap_or( "N/A" )
            );
        }

        self.ai.app.get_log_mut().end( "" );
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
    fn chat
    (
        &mut self,
        /* Prompt */
        prompt: &str
    )
    {
        let api_url = get_api_url( self.ai, &self.name );
        let token = get_token( self.ai );
        let model = self.ai.get_model();
        let client = self.create_client();

        /* Trigger before request event */
        self.ai.on_before_request
        (
            &prompt,
            &self.name,
            &model,
            &api_url
        );

        /* Prepare request */
        let mut payload = serde_json::json!
        (
            {
                "messages": [{ "role": "user", "content": prompt }],
                "model": model,
            }
        );

        /* Providers specific */
        match self.ai.get_provider().as_str()
        {
            "deepseek" =>
            {
                let think = self.ai.get_config_val( &[ "think" ], false );
                if !think
                {
                    payload[ "thinking" ] = serde_json::json!({ "type": "disabled" });
                }
            }
            _ => {}
        }

        self.ai.app.get_log_mut().dump( "Request", &payload.to_string() );

        /*
            Request
        */
        let response = if token.is_empty()
        {
            client.post( &api_url )
            .header( "Content-Type", "application/json" )
            .json( &payload )
            .send()
        }
        else
        {
            client.post( &api_url )
            .bearer_auth( &token )
            .header( "Content-Type", "application/json" )
            .json( &payload )
            .send()
        };



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
                self.ai.on_after_response
                (
                    &full_answer,
                    &self.name,
                    &model,
                    &api_url,
                    "chat"
                );

                /* Get openai fields*/
                let
                (
                    error,
                    _, //think,
                    content,
                    _, //prompt_tokens,
                    _, //answer_tokens,
                    result
                ) = self.parse_response( full_answer );

                if result
                {
                    self.ai.handle_chat_response( &content )
                }
                else
                {
                    /* Error response from API */
                    let message = if error.is_empty()
                    {
                        content
                    }
                    else
                    {
                        format!( "{}\n{}", error, content )
                    };
                    println!( "{}", message );
                }
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
                    "API error\n{}",
                    &e.to_string()
                );

                let provider_name = self.get_name().to_string();
                let proxy = self.ai.read_proxy();
                self.ai.app.get_log_mut()
                .error( "API error" )
                .prm( "error", &e.to_string() )
                .prm( "provider", provider_name )
                .prm( "api", api_url )
                .prm( "proxy", proxy )
                ;
            }
        }
    }
}
