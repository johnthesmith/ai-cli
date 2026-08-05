/*
    SPDX-License-Identifier: MIT
    SPDX-FileCopyrightText: 2026 Still Swamp
*/

/*
    Model secion
*/

impl Ai
{
    /*
        Return file for current model
        Placeholders: %profile%, %provider%, %chat%
    */
    fn get_model_file_path( &self )
    -> String
    {
        core::expand_path
        (
            &self.get_config_str
            (
                &[ "model-file" ],
                "%profile-path%/chats/%chat%/models/%provider%.txt"
            )
            .replace( "%profile-path%", &self.get_profile_path() )
            .replace( "%profile%", &self.get_profile() )
            .replace( "%provider%", &self.get_provider() )
            .replace( "%chat%", &self.get_chat() )
        )
    }



    pub fn read_model( &self ) -> String
    {
        let path = self.get_model_file_path();

        if let Ok(content) = std::fs::read_to_string( &path )
        {
            let model = content.trim().to_string();
            if !model.is_empty()
            {
                return model;
            }
        }
        "default".to_string()
    }



    /*
        Change current model
    */
    fn bind_model
    (
        &mut self,
        id: &str
    )
    -> &mut Self
    {
        let file_path = self.get_model_file_path();

        if let Err(e) = core::ensure_directory( &file_path )
        {
            self.app.get_log_mut()
            .error( "Failed to ensure model directory" )
            .prm( "error", &e );
            return self;
        }

        if let Err(e) = std::fs::write( &file_path, id )
        {
            self.app.get_log_mut()
            .error( "Failed to bind model" )
            .prm( "path", &file_path )
            .prm( "error", &e.to_string() );
        }
        else
        {
            self.app.get_log_mut()
            .trace( "Model binded" )
            .prm( "id", id);
        }

        self
    }



    /*
        Return list of model aliases for current provider
    */
    fn get_models
    (
        &self,
        provider: &str
    )
    -> Vec<String>
    {
        self.app.config
        [ "application" ]
        [ "ai" ]
        [ "providers" ]
        [ &provider ]
        [ "models" ]
        .as_object()
        .map( |obj| obj.keys().map( |k| k.to_string()).collect())
        .unwrap_or_default()
    }



    /*
        Return model
    */
    fn get_model( &self )
    -> String
    {
        self.model.clone()
    }



    /*
        Return safe model (replace special chars for filesystem)
    */
    fn get_model_safe( &self )
    -> String
    {
        self.get_model()
        .replace( '/', "_" )
        .replace( '\\', "_" )
        .replace( '.', "_" )
        .replace( "..", "_" )
    }



    /*
        Set modle for current session
    */
    fn set_model
    (
        &mut self,
        name: &str
    ) -> &mut Self
    {
        self.model = name.to_string();
        self
    }



    /*
        Return return model name by current model alias
    */
    fn get_model_name( &self )
    -> String
    {
        self.app.config
        [ "application" ]
        [ "ai" ]
        [ "providers" ]
        [ self.get_provider() ]
        [ "models" ]
        [ self.get_model() ].get_str( "unknown" )
    }
}
