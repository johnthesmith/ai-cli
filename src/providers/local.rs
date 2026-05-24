use crate::Ai;
use crate::ai::response::{ChatResponse, SummaryResponse};
use super::Provider;

/*
    Local Provider for Ollama API.
    API: http://localhost:11434/api/generate
*/
pub struct LocalProvider<'a>
{
    /* Shared reference to AI instance (config, token, logs) */
    ai: &'a mut Ai,
    /* Provider name (github, openai, deepseek, groq, together) */
    name: String
}



impl<'a> LocalProvider<'a>
{
    /*
        Create new default provider instance.
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
            name: "local".to_string(),
        }
    }
}



impl<'a> Provider for LocalProvider<'a>
{
    fn get_name( &self ) -> &str
    {
        "local"
    }

    fn chat( &mut self ) 
    {
    }

    fn summary( &mut self )
    {
    }
}
