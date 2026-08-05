/*
    Ai implementation
*/
impl Ai
{
    /*
        ping and return AI
    */
    pub fn create() -> Self
    {
        let app = App::create();
        let log = app.get_log_rc();

        Self
        {
            app,
            profile: "default".to_string(),

            fact_delimiter: "#FACT".to_string(),

            provider: String::new(),
            model: String::new(),
            chat: String::new(),
            memory_id: String::new(),
            prompt_id: String::new(),

            storage: Storage::new( log.clone() ),
            fact_db: Storage::new( log.clone() ),

            status: Vec::new(),

            access_access: "s".to_string(),
            access_history: "si".to_string(),
            access_memory: "s".to_string(),
            access_prompt: "s".to_string(),
            access_shell: "i".to_string(),
            access_clipboard: "i".to_string(),
            access_read: "s".to_string(),
            access_write: "su".to_string(),

            no_history: false,
            no_memory: false,
            no_prompt: false,

            /* Create map write translation */
            write_translation: BTreeMap::new(),

            colorize: true
        }
    }
}
