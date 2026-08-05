/*
    SPDX-License-Identifier: MIT
    SPDX-FileCopyrightText: 2026 Still Swamp
*/


impl Ai
{
    /*
        Build yaml with session information and return it in to stdout
    */
    fn out_info( &mut self )
    -> &mut Self
    {
        let info = json!
        (
            {
                "version": self.get_version(),
                "session":
                {
                    "profile": self.get_profile(),
                    "provider": self.get_provider(),
                    "model": self.get_model(),
                    "model-name": self.get_model_name(),
                    "chat": self.get_chat(),
                    "memory-id": self.get_memory_id(),
                    "prompt-id": self.get_prompt_id(),
                    "proxy":  self.read_proxy(),
                    "fact-delimiter": self.fact_delimiter
                },
                "access":
                {
                    "access": self.access_to_str( &self.access_access ),
                    "history": self.access_to_str( &self.access_history ),
                    "memory": self.access_to_str( &self.access_memory ),
                    "prompt": self.access_to_str( &self.access_prompt ),
                    "shell": self.access_to_str( &self.access_shell ),
                    "clipboard": self.access_to_str( &self.access_clipboard ),
                    "read": self.access_to_str( &self.access_read ),
                    "write": self.access_to_str( &self.access_write )
                },
                "files":
                {
                    "log": self.get_app().get_log().get_file_path(),
                    "config": self.get_config_file(),

                    "chat-file": self.get_chat_file(),
                    "chats-path": self.get_chats_path(),
                    "chat-path": self.get_chat_path( &self.get_chat() ),

                    "profile-file": self.get_profile_file(),
                    "profiles-path": self.get_profiles_path(),
                    "profile-path": self.get_profile_path(),
                    "provider": self.get_provider_file(),
                    "prompt": self.get_prompt_file( &self.get_chat() ),
                    "prompt-content": self.get_prompt_content_file(),
                    "prompts-path": self.get_prompts_path( &self.get_chat() ),

                    "model": self.get_model_file_path(),

                    "memory-path": self.get_memory_path(),
                    "memory-of-chat": self.get_memory_of_chat_file(),
                    "memory": self.get_memory_file(),

                    "token": self.get_token_path(),
                    "history": self.get_history_file_path(),
                },
                "statistics":
                {
                    "max_prompt_size_bytes": self.get_max_prompt_bytes(),
                    "history_size_bytes": self.get_history_file_size(),
                    "memory_size_bytes": self.get_memory_file_size(),
                    "prompt_size_bytes": self.get_prompt_content_file_size()
                },
                "state":
                {
                    "code": self.app.state.get_code(),
                    "details": self.app.state.get_details()
                }
            }
        );

        println!( "{}", serde_yaml::to_string(&info).unwrap_or_default() );

        self
    }
}
