use crate::Ai;
use core::expand_path;


/*
    Return API endpoint URL for the provider.
*/
pub fn get_api_url
(
    ai: &Ai, 
    provider_name: &str
)
-> String
{
    ai.application.config
        .as_ref()
        .and_then
        (
            |cfg| cfg
            ["application"]
            ["ai"]
            ["providers"]
            [provider_name]
            ["api"]
            .as_str()
        )
        .map(|s| s.to_string())
        .unwrap_or_default()
}



/*
    Return authentication token for the provider.
*/
pub fn get_token
(
    ai: &Ai
)
-> String 
{
    let token_path = ai.get_token_path();

    if token_path.is_empty() 
    {
        String::new()
    }
    else
    {
        let expanded_path = expand_path( &token_path );
        std::fs::read_to_string(&expanded_path)
            .ok()
            .map(|s| s.trim().to_string())
            .unwrap_or_default()
    }
}
