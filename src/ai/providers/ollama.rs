/*
    SPDX-License-Identifier: MIT
    SPDX-FileCopyrightText: 2026 Still Swamp
*/



use reqwest::blocking::Response;
use core::Color;
use crate::Ai;
use super::api::get_api_url;
use super::Provider;



/*
    Ollama Provider for local LLM.
    API: http://localhost:11434/api/generate
*/
pub struct OllamaProvider<'a>
{
    /* Shared reference to AI instance (config, token, logs) */
    ai: &'a mut Ai,
    /* Provider name (github, openai, deepseek, groq, together) */
    name: String,
}



impl<'a> OllamaProvider<'a>
{
    /*
        Create new ollama provider instance.
    */
    pub fn new
    (
        /* AI application instance (shared ownership) */
        ai: &'a mut Ai
    )
    -> Self
    {
        Self
        {
            ai: ai,
            name: "ollama".to_string()
        }
    }



    fn create_client(&self) -> reqwest::blocking::Client
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



    fn parse_ollama_response
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
        /* Ollama doesn't support think tag */
        let think = String::new();
        let content;
        let mut prompt_tokens = 0;
        let mut completion_tokens = 0;
        let success;

        let raw = raw.trim();

        /* Get json from raw answer */
        match serde_json::from_str::<serde_json::Value>(&raw)
        {
            Err(e) =>
            {
                self.ai.app.get_log_mut()
                    .error("Failed to parse Ollama response")
                    .prm("error", &e.to_string())
                    .prm("content", &raw);

                error_msg = e.to_string();
                content = raw.to_string();
                success = false;
            }

            Ok(json) =>
            {
                if let Some(error) = json.get("error") {
                    error_msg = error.as_str()
                        .unwrap_or("Unknown Ollama error")
                        .to_string();
                    content = error_msg.clone();
                    success = false;
                } 
                else
                {
                    /* Ollama returns response in "response" field */
                    content = json
                        .get("response")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .trim()
                        .to_string();

                    /* Ollama token counts */
                    prompt_tokens = json
                        .get("prompt_eval_count")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);

                    completion_tokens = json
                        .get("eval_count")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);

                    success = true;
                }
            }
        }
        (error_msg, think, content, prompt_tokens, completion_tokens, success)
    }



    /*
       Dump response headers.
    */
    fn dump_headers(&mut self, resp: &Response)
    {
        self.ai.app.get_log_mut().begin("Response headers");

        for (name, value) in resp.headers().iter()
        {
            self.ai.app.get_log_mut().trace("").prm
            (
                name.as_str(),
                value.to_str().unwrap_or("N/A")
            );
        }

        self.ai.app.get_log_mut().end("");
    }
}



impl<'a> Provider for OllamaProvider<'a>
{
    /*
        Return provider name identifier.
    */
    fn get_name(&self) -> &str
    {
        "ollama"
    }



    /*
        Send chat request and parse response.
    */
    fn chat
    (
        &mut self,
        prompt: &str
    )
    {
        let name = self.get_name().to_string();

        let api_url = get_api_url( self.ai, &name );
        let model = self.ai.get_model();
        let client = self.create_client();

        /* Trigger before request event */
        self.ai.on_before_request(&prompt, &name, &model, &api_url );
    
        /* Prepare request for Ollama API */
        let payload = serde_json::json!
        (
            {
                "model": model,
                "prompt": prompt,
                "stream": false
            }
        );

        /*
            Request
        */
        let response = client.post( &api_url )
        .header("Content-Type", "application/json")
        .json(&payload)
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
                    _, //think,
                    content,
                    _, //prompt_tokens,
                    _, //answer_tokens,
                    result
                ) = self.parse_ollama_response( full_answer );

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
            Err(e) =>
            {
                /* event */
                self.ai.on_after_response
                (
                    &e.to_string(),
                    &name,
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

                let proxy = self.ai.read_proxy();
                self.ai.app.get_log_mut()
                .error( "API error" )
                .prm( "error", &e.to_string() )
                .prm( "provider", &name )
                .prm( "api", api_url )
                .prm( "proxy", proxy );
            }
        }      
    }
}
