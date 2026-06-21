/*
    SPDX-License-Identifier: MIT
    SPDX-FileCopyrightText: 2026 Still Swamp
*/



use regex::Regex;
use reqwest::blocking::Response;

use crate::Ai;
use super::api::{ get_api_url, get_token };
use super::Provider;
use core::SerdeExt;

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



fn replace_placeholders
(
    value: &mut serde_json::Value,
    prompt: &str,
    model: &str
)
{
    match value
    {
        serde_json::Value::String(s) =>
        {
            *s = s.replace("%prompt%", prompt).replace("%model-name%", model);
        }
        serde_json::Value::Object(obj) =>
        {
            for v in obj.values_mut()
            {
                replace_placeholders(v, prompt, model);
            }
        }
        serde_json::Value::Array(arr) =>
        {
            for v in arr
            {
                replace_placeholders(v, prompt, model);
            }
        }
        _ => {}
    }
}




fn select_rule
(
    api_formats: &serde_json::Value,
    provider_name: &str,
    model: &str
) -> serde_json::Value
{
    if api_formats.is_null()
    {
        return serde_json::Value::Null;
    }

    if let Some(formats) = api_formats.as_array()
    {
        for rule in formats
        {
            let rule_provider = rule["provider"]
                .as_str()
                .unwrap_or("*");
            let rule_model = rule["model"]
                .as_str()
                .unwrap_or("*");

            if
            (
                rule_provider == "*" ||
                rule_provider == provider_name
            )
            &&
            (
                rule_model == "*" ||
                rule_model == model
            )
            {
                return rule.clone();
            }
        }
    }

    serde_json::Value::Null
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
        .timeout
        (
            std::time::Duration::from_millis
            ( self.ai.get_request_timeout_ms() )
        )
        .danger_accept_invalid_certs( true )
        .connect_timeout
        (
            std::time::Duration::from_millis
            (
                self.ai.get_connect_timeout_ms()
            )
        );

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
        /* Answer */
        raw: String,
        /* Answer rule */
        rule: &serde_json::Value
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
        let mut content = String::new();
        let mut prompt_tokens = 0;
        let mut completion_tokens = 0;
        let mut success = false;

        let raw = raw.trim();

        /* Split think and raw */
        let re = Regex::new( r"(?s)<think>(.*?)</think>" ).unwrap();
        let think = re.captures( raw )
        .and_then( |cap| cap.get( 1 )
        .map( |m| m.as_str().to_string() ))
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
                    let answer = &rule[ "answer" ];
                    let mut source = "";

                    if let Some( arr ) = answer.as_array()
                    {
                        let mut current = &json;
                        for key in arr
                        {
                            if let Some(idx) = key.as_u64()
                            {
                                if let Some(next) =
                                current
                                .as_array()
                                .and_then(|a| a.get(idx as usize))
                                {
                                    current = next;
                                }
                                else
                                {
                                    break;
                                }
                            }
                            else if let Some(key_str) = key.as_str()
                            {
                                if let Some(next)
                                = current
                                .as_object()
                                .and_then(|o| o.get(key_str))
                                {
                                    current = next;
                                }
                                else
                                {
                                    break;
                                }
                            }
                        }
                        source = current.as_str().unwrap_or("");
                    }

                    if !source.is_empty()
                    {
                        content = source
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
                    else
                    {
                        let answer_vec = answer.get_array(vec![]);
                        let path_str = answer_vec
                        .iter()
                        .map
                        (
                            |v|
                            {
                                if let Some(s) = v.as_str()
                                {
                                    s.to_string()
                                }
                                else if let Some(n) = v.as_u64()
                                {
                                    n.to_string()
                                }
                                else
                                {
                                    "".to_string()
                                }
                            }
                        )
                        .collect::<Vec<String>>()
                        .join( "," );

                        self.ai.app.state.set_state
                        (
                            "answer-source-is-empty",
                            serde_json::json!
                            (
                                {
                                    "path": path_str
                                }
                            )
                        );
                    }
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
        let provider_name = self.get_name().to_string();
        let model_name = self.ai.get_model_name();
        let model = self.ai.get_model();
        let client = self.create_client();


        /* Trigger before request event */
        self.ai.on_before_request
        (
            &prompt,
            &self.name,
            &model_name,
            &api_url
        );

        let api_formats = &self.ai.app.config
        [ "application" ]
        [ "ai" ]
        [ "api-format" ];

        let selected_rule = select_rule(api_formats, &provider_name, &model);
        if selected_rule.is_null()
        {
            self.ai.app.state.set_state
            (
                "rule-not-found",
                serde_json::json!
                ({
                    "model": &model,
                    "provider": &provider_name
                })
            );
        }
        else
        {
            let mut payload =
            {
                let p = serde_json::json!({ "model": model });
                let request_json = serde_json::to_value
                (
                    &selected_rule
                    ["request"]
                )
                .unwrap();
                p.merge( &request_json )
            };

            replace_placeholders( &mut payload, &prompt, &model_name );

            /* Dump payload to log */
            self.ai.app.get_log_mut().dump
            (
                "Request",
                &serde_json::to_string_pretty( &payload ).unwrap()
             );

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
                        &model_name,
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
                    ) = self.parse_response
                    (
                        full_answer,
                        &selected_rule
                    );

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
                    let proxy = self.ai.read_proxy();
                    let provider_name = self.get_name().to_string();
                    let error_msg = format!( "{:#?}", e );

                    /* event */
                    self.ai.on_after_response
                    (
                        &error_msg,
                        &self.name,
                        &model_name,
                        &api_url,
                        "chat"
                    );

                    self.ai.app.state.set_state
                    (
                        "api-error",
                        serde_json::json!
                        (
                            {
                                "message": &error_msg,
                                "provider": provider_name,
                                "api": api_url,
                                "proxy": proxy
                            }
                        )
                    );

                    self.ai.app.get_log_mut()
                    .error( "API error" )
                    .prm( "error", &error_msg )
                    .prm( "provider", provider_name )
                    .prm( "api", api_url )
                    .prm( "proxy", proxy )
                    ;
                }
            }
        }
    }
}
