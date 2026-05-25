mod default;
// mod local;
// mod anthropic;
mod api;

pub use default::OpenAICompatibleProvider;
// pub use local::LocalProvider;
// pub use anthropic::AnthropicProvider;

use crate::Ai;



/*
    Provider trait - interface for AI service implementations.
    Each provider must implement HTTP request building and response parsing.
*/
pub trait Provider
{
    /*
        Return provider name identifier (e.g., "github", "openai").
    */
    fn get_name( &self ) -> &str;



    /*
        Send chat request and parse response.
        Returns structured ChatResponse with command, message, buffer, memory, tokens.
    */
    fn chat( &mut self );



    /*
        Send summarization request and parse response.
        Returns structured SummaryResponse with summary text and tokens.
    */
    fn summary
    (
        &mut self,
        /* Packing percent from 0 to 100 ( all will be packing ) */ 
        percent: u64
    );
}



/*
    Factory method to create provider instance with AI context.
*/
pub fn create_provider<'a>
(
    /* Provider name: "github", "openai", "deepseek", etc. */
    name: &str,
    /* AI application instance (shared ownership) */
    ai: &'a mut Ai
)
/* Boxed trait object implementing Provider */
-> Box<dyn Provider +'a>
{
    match name
    {
        // OpenAI-compatible providers (same API format)
        "github" | "openai" | "deepseek" | "groq" | "together" =>
            Box::new( OpenAICompatibleProvider::new( name, ai )),

        // Local Ollama (different API format)
//        "local" => Box::new( LocalProvider::new( ai )),

        // Anthropic Claude (different API format)
//        "anthropic" => Box::new( AnthropicProvider::new( ai )),

        _ => unimplemented!(),
    }
}
