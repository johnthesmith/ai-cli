use crate::Ai;
use crate::ai::response::{ChatResponse, SummaryResponse};
use super::Provider;



pub struct AnthropicProvider<'a>
{
    /* Shared reference to AI instance (config, token, logs) */
    ai: &'a mut Ai,
    /* Provider name (github, openai, deepseek, groq, together) */
    name: String,
}



impl<'a> AnthropicProvider<'a>
{
    pub fn new(ai: &'a mut Ai ) -> Self
    {
        Self 
        {
            ai: ai,
            name: "anthropic".to_string()
        }
    }
}



impl<'a> Provider for AnthropicProvider<'a>
{
    fn get_name( &self ) -> &str
    {
        "anthropic"
    }



    fn chat( &mut self )
    {
    }



    fn summary( &mut self )
    {
    }
}
