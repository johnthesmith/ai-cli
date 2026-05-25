/*
    AI response structure for chat completions.
    Parsed from provider JSON responses into a unified format.
*/

#[derive(Debug, Clone, Default)]
pub struct ChatResponse 
{
    /* Command to inject into TTY input buffer (keyboard simulation) */
    pub command: String,  
    /* Output message to display to user (stdout or out destination) */
    pub message: String,
    /* Data to copy to system clipboard */
    pub clipboard: String,
    /* Thinking/reasoning content from AI (if supported) */
    #[allow(dead_code)]
    pub think: String,
    /* Buffer content to write to buffer file and forward to stdout */
    pub buffer: String,
    /* Memory facts to store in chat memory */
    pub memory: String,
    /* Number of tokens in user prompt (from provider response) */
    pub prompt_tokens: u64,
    /* Number of tokens in AI answer (from provider response) */
    pub answer_tokens: u64,
}



/*
    Summary response structure.
*/
//#[derive(Debug, Clone, Default)]
//pub struct SummaryResponse
//{
//    /* Summarized text */
//    pub summary: String,
//    /* Thinking/reasoning content from AI (if supported) */
//    pub think: String,
//    /* Token usage */
//    pub prompt_tokens: u64,
//    pub answer_tokens: u64,
//    /* Success status */
//    pub success: bool,
//}
