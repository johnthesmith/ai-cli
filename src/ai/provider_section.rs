/*
    SPDX-License-Identifier: MIT
    SPDX-FileCopyrightText: 2026 Still Swamp
*/

/*
    Provider section
*/

impl Ai
{
    /*
        Return provider
    */
    fn get_provider( &self )
    -> String
    {
        self.provider.clone()
    }



    /*
        Set provider for current session
    */
    fn set_provider
    (
        &mut self,
        name: &str
    )
    -> &mut Self
    {
        self.provider = name.to_string();
        self
    }



    /*
        Return list of provider names
    */
    fn get_providers( &self )
    -> Vec<String>
    {
        self.app.config
        [ "application" ]
        [ "ai" ]
        [ "providers" ]
        .as_object()
        .map(|obj| obj.keys().map(|k| k.to_string()).collect())
        .unwrap_or_default()
    }



    /*
        Return provider file path
    */
    fn get_provider_file( &self ) -> String
    {
        core::expand_path
        (
            &self.get_config_str
            (
                &[ "provider-file" ],
                "%profile-path%/chats/%chat%/provider.txt"
            )
            .replace( "%profile-path%", &self.get_profile_path())
            .replace( "%profile%", &self.get_profile() )
            .replace( "%chat%", &self.get_chat() )
        )
    }



    /*
        Check provider exists at config file
    */
    fn provider_exists
    (
        &self,
        /* Provider id */
        id: &str
    )
    -> bool
    {
        let config = &self.app.config;
        let provider = &config[ "application" ][ "ai" ][ "providers" ][ id ];
        !provider.is_null()
    }



    /*
        Return provider
    */
    fn read_provider( &self )
    -> String
    {
        let path = self.get_provider_file();

        if let Ok( content ) = std::fs::read_to_string( &path )
        {
            let provider = content.trim().to_string();
            if !provider.is_empty()
            {
                return provider;
            }
        }

        "deepseek".to_string()
    }



    /*
        Change current provider
    */
    fn bind_provider
    (
        &mut self,
        new_provider: &str
    ) -> &mut Self
    {
        let file_path = self.get_provider_file();

        if let Err( e ) = core::ensure_directory( &file_path )
        {
            self.app.get_log_mut()
            .error( "Failed to ensure provider directory" )
            .prm( "error", &e );
            return self;
        }

        if let Err( e ) = std::fs::write( &file_path, new_provider )
        {
            self.app.get_log_mut()
            .error( "Failed to bind provider" )
            .prm( "path", &file_path )
            .prm( "error", &e.to_string() );
        }
        else
        {
            self.app.get_log_mut()
            .trace( "Provider binded" )
            .prm( "provider", new_provider );
        }

        self
    }



    /*
        Event triggered before sending HTTP request to LLM provider.
        Logs the prompt for debugging and audit purposes.
    */
    pub fn on_before_request
    (
        &mut self,
        /* Full prompt text that will be sent to LLM */
        prompt: &str,
        /* Provider name */
        provider: &str,
        /* Model identifier */
        model: &str,
        /* API endpoint URL for the request */
        api_url: &str
    )
    {
        self.app.get_log_mut()
        .dump( "Prompt to LLM", prompt )
        .prm( "provider", provider )
        .prm( "model", model )
        .prm( "api", api_url )
        ;
    }



    /*
        Event triggered after receiving HTTP response from LLM provider.
        Logs the response for debugging and audit purposes.
    */
    pub fn on_after_response
    (
        &mut self,
        /* Raw response text from LLM */
        response: &str,
        /* Provider name ( "deepseek", "openai", ... ) */
        provider: &str,
        /* Model identifier used for the request */
        model: &str,
        /* API endpoint URL used for the request */
        api_url: &str,
        /* Promt id */
        prompt: &str
    )
    {
        self.app.get_log_mut()
        .dump( "Response from LLM", response )
        .prm( "provider", provider )
        .prm( "model", model )
        .prm( "api", api_url )
        .prm( "promtp", prompt );
    }
}
