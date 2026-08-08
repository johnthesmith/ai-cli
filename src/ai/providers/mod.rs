/*
    SPDX-License-Identifier: MIT
    SPDX-FileCopyrightText: 2026 Still Swamp
*/



mod default;
pub mod api;

pub use default::OpenAICompatibleProvider;

use crate::Ai;



/*
    Provider trait - interface for AI service implementations.
    Each provider must implement HTTP request building and response parsing.
*/
pub trait Provider
{
    /*
        Return provider name identifier (e.g., "deepseek", "openai" ).
    */
    fn get_name( &self ) -> &str;



    /*
        Send chat request and parse response.
        Returns structured ChatResponse with command, message, pool, memory, tokens.
    */
    fn chat
    (
        &mut self,
        prompt: &str
    );
}



/*
    Factory method to create provider instance with AI context.
*/
pub fn create_provider<'a>
(
    /* Provider name: deepseek, openai, etc. */
    name: &str,
    /**/
    /* AI application instance (shared ownership) */
    ai: &'a mut Ai
)
/* Boxed trait object implementing Provider */
-> Box<dyn Provider +'a>
{
    /* Type of api */
    let api_type = "openai";
    match api_type
    {
        /* OpenAI-compatible providers (same API format) */
        _
        => Box::new( OpenAICompatibleProvider::new( name, ai ))
    }
}
