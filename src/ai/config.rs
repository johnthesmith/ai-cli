/*
    SPDX-License-Identifier: MIT
    SPDX-FileCopyrightText: 2026 Still Swamp
*/

include!( "config_template.rs" );


/*
    Config
*/

impl Ai
{
    /*
        Return config file path for current profile
    */
    fn get_config_file( &self )
    -> String
    {
        core::expand_path( &( self.get_profile_path() + "/config.yaml" ))
    }



    /*
        Generate config file
    */
    fn generate_config( &mut self )
    -> &mut Self
    {
        let path = self.get_config_file();

        if let Err(e) = core::ensure_directory( &path )
        {
            self.app.state.set_state
            (
                "config-dir-create-error",
                json!({ "error": e.to_string(), "path": path })
            );
            return self;
        }

        match std::fs::write(&path, CONFIG.as_bytes())
        {
            Ok(_) =>
            {
                self.app.state = State::ok();
            }
            Err(e) =>
            {
                self.app.state.set_state
                (
                    "config-write-error",
                    json!({ "error": e.to_string(), "path": path })
                );
            }
        }

        self
    }



    /*
        Return request timeout in milliseconds
    */
    fn get_request_timeout_ms( &self ) -> u64
    {
        self . get_config_int( &[ "request-timeout-ms" ], 30000 )
    }



    /*
        Return connect timeout in milliseconds
    */
    fn get_connect_timeout_ms( &self ) -> u64
    {
        self.get_config_int( &[ "connect-timeout-ms" ], 10000 )
    }



    fn select_rule
    (
        &self,
        provider_name: &str,
        model: &str
    )
    -> serde_json::Value
    {
        let config = &self.app.config;
        let api_formats = &config["application"]["ai"]["rules"];

        if api_formats.is_null()
        {
            return serde_json::Value::Null;
        }

        if let Some(formats) = api_formats.as_array()
        {
            for rule in formats
            {
                let rule_provider = rule["provider"]
                    .as_str()
                    .unwrap_or("*");
                let rule_model = rule[ "model" ]
                    .as_str()
                    .unwrap_or("*");

                if
                (
                    rule_provider == "*" ||
                    rule_provider == provider_name
                )
                &&
                (
                    rule_model == "*" ||
                    rule_model == model
                )
                {
                    return rule.clone();
                }
            }
        }

        serde_json::Value::Null
    }



    fn get_config_value
    (
        &self,
        keys: &[&str]
    )
    -> JsonValue
    {
        let config = &self.app.config;
        let provider = self.get_provider();
        let model = self.get_model();

        // 1. Ищем в правиле
        let rule = self.select_rule(&provider, &model);
        if !rule.is_null()
        {
            let mut current = &rule;
            let mut found = true;
            for &k in keys
            {
                if let Some(next) = current.get(k)
                {
                    current = next;
                }
                else
                {
                    found = false;
                    break;
                }
            }
            if found
            {
                return current.clone();
            }
        }

        let mut current = config;
        for &k in keys
        {
            if let Some( next ) = current.get(k)
            {
                current = next;
            } else {
                return JsonValue::Null;
            }
        }

        current.clone()
    }



    fn get_config_bool
    (
        &self,
        keys: &[&str],
        default: bool
    )
    -> bool
    {
        self.get_config_value( keys ).get_bool( default )
    }



    fn get_config_str
    (
        &self,
        keys: &[&str],
        default: &str
    )
    -> String
    {
        self.get_config_value(keys).get_str(default)
    }



    fn get_config_int
    (
        &self,
        keys: &[&str],
        default: u64
    )
    -> u64
    {
        self.get_config_value( keys ).get_int( default )
    }



    fn get_config_size
    (
        &self,
        keys: &[&str],
        default: usize
    ) -> usize
    {
        self.get_config_value( keys ).get_size( default )
    }



    /*
        Return proxy for current provider
    */
    fn read_proxy( &self )
    -> String
    {
        self.get_config_str( &[ "proxy" ], &String::new() )
    }
}
