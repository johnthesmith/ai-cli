/*
    SPDX-License-Identifier: MIT
    SPDX-FileCopyrightText: 2026 Still Swamp
*/

impl Ai
{
    /***************************************************************************
        Completion
    */

    /*
        Bash completion handler
    */
    pub fn completion
    (
        &mut self,
        comp_line : &str
    )
    {
        /* Retrieve cursor point */
        let comp_point = self.app.config[ "comp-point" ].get_int( 0 ) as usize;

        /* Cut line at cursor point */
        let end =
        if
            comp_point <= comp_line.len()
            && comp_line.is_char_boundary(comp_point)
        {
            comp_point
        }
        else
        {
            let mut end = comp_point;
            while end > 0 && !comp_line.is_char_boundary(end)
            {
                end -= 1;
            }
            end
        };

        let line = &comp_line[..end];
        let parts: Vec<&str> = line.split( ' ' ).collect();
        let current = *parts.last().unwrap_or(&"");

        /* Extract key value from current part */
        let ( key, val, eq ) = if let Some( eq_pos ) = current.find( '=' )
        {
            ( &current[..eq_pos], &current[eq_pos + 1..], true )
        }
        else
        {
            ( current, "", false )
        };

        /* Return keys by current key*/
        let keys = self.find_keys( key, &comp_line );

        /* Define finel vector */
        let mut list = Vec::<String>::new();

        match keys.len()
        {
            0 => {}
            1 =>
            {
                /* Key founded */
                if key == keys[0]
                {
                    /* Key complete and need values */
                    let values = self.find_values( key, val, &comp_line );
                    match values.len()
                    {
                        0 =>
                        {
                            /* One value founded */
                            if eq
                            {
                            }
                            else
                            {
                                /* No values founded */
                                list.push( format!( "{} ", key ));
                            }
                        }
                        1 =>
                        {
                            /* One value founded */
                            if eq
                            {
                                list.push( format!( "{}", values[0] ));
                            }
                            else
                            {
                                list.push( format!( "{}=", key ));
                            }
                        }
                        _ =>
                        {
                            if eq
                            {
                                /* All values dump */
                                list = values.clone();
                            }
                            else
                            {
                                /* Many values but not = then set =*/
                                list.push( format!( "{}=", key ));
                            }
                        }
                    }
                }
                else
                {
                    /* Key not fully typed — complete the key */
                    list.push(keys[0].clone());
                }
            }
            _ =>
            {
                /* All posible keys */
                list = keys.clone();
            }
        }

        for l in list
        {
            println!( "{}", l );
        }

    }



    /*
        Filter values by prefix
        Used for file completion and other dynamic completions
    */
    fn compl_filter_by_key
    (
        &self,
        values: Vec<String>,
        val: &str
    )
    -> Vec<String>
    {
        values
        .iter()
        .filter(|v| v.starts_with(val))
        .map(|v| v.to_string())
        .collect()
    }



    /*
        Returns a list of matching keys for the given partial key.
    */
    fn find_keys
    (
        &mut self,
        key: &str,
        comp_line: &str
    )
    -> Vec<String>
    {
        let mut result = Vec::new();

        /* Help & Info */
        [
            "-?",
            "-h",
            "--help",
            "-v",
            "--version"
        ]
        .iter().for_each( |k| result.push( k.to_string() ) );

        /* Init */
        let path = self.get_profiles_path();
        if path.is_empty()
        {
            result.push( "--init".to_string());
        }
        else
        {
            [
                "--info", "-i",

                /* Session control */
                "--no-prompt",
                "--no-command",
                "--color",
                "--status",
                "--set",

                /* Files */
                "--read",
                "--write",
                "--restore",

                /* Profile & provider & model & chat (temporary) */
                "--profile",
                "--provider",
                "--model",
                "--chat",
                "--memory",
                "--prompt",

                /* Permanent bind */
                "--bind-profile",
                "--bind-provider",
                "--bind-prompt",
                "--bind-model",
                "--bind-chat",
                "--bind-memory",

                // LLM access rights (iud)
                "--access-history",
                "--access-memory",
                "--access-prompt",

                /* Storage operations: history */
                "-rmh",
                /* history operations  */
                "--out-history", "-oh",
                "--remove-history", "-rh",
                /* memory  operations */
                "--out-memory", "-om",
                "--remove-memory", "-rm",
                /* prompt operations */
                "--out-prompt", "-op",
                "--out-prompt-content", "-opc",
                "--build-prompt",

                /* Fact operations */
                "--select-fact",
                "--delete-fact",
                "--update-fact",
                "--insert-fact",

                /* Specific features */
                "--comp-line",
                "--comp-point",

                /* Specific features */
                "--tiocsti",

            ].iter().for_each(|k| result.push( k.to_string() ));
        }

        /* Actor & body for insert/update */
        let update_keys = [ "--update", "--insert" ];
        let show_actor_body = update_keys
        .iter()
        .any(|k| comp_line.contains( k ));

        if show_actor_body
        {
            [ "--actor", "--body", "--domain" ]
            .iter().for_each(|k| result.push( k.to_string() ));
        }

        self.compl_filter_by_key( result, key )
    }



    /*
        Return list of values for key
    */
    pub fn find_values
    (
        &mut self,
        key: &str,
        val: &str,
        comp_line: &str
    )
    -> Vec<String>
    {
        let values = match key
        {
            "--read" | "--write" | "--restore" =>
            {
                self.get_files( val )
            }

            "--set" =>
            {
                self.app.get_sets()
            }

            "--chat" | "--bind-chat" =>
            {
                self.get_chats()
            }

            "--provider" | "--bind-provider" =>
            {
                self.get_providers()
            }

            "--model" | "--bind-model" =>
            {
                let provider = self.extract_provider_from_line(comp_line);
                self.get_models(&provider)
            }

            "--memory" | "--bind-memory" =>
            {
                self.get_memories()
            }

            "--build-prompt" =>
            {
                self.get_prompt_templates()
            }

            "--prompt" | "--bind-prompt"  =>
            {
                let chat = self.extract_chat_from_line( comp_line );
                self.get_prompts( &chat )
            }

            "--access-history" |
            "--access-memory" |
            "--access-prompt" =>
            {
                self.get_complete_value_access()
            }

            "--update-fact" | "--delete-fact" | "--select-fact" =>
            {
                self.compile_prompt( "", "" );
                self.storage.get_id_list()
            }

            "--color" | "--status" =>
            {
                return vec![ "true".to_string(), "false".to_string() ];
            }

            _ =>
            {
                return vec![];
            }
        };

        self.compl_filter_by_key(values, val)
    }



    /*
        Return autocomplete for access
    */
    fn get_complete_value_access( &self )
    -> Vec<String>
    {
        [
            "s", "u", "i", "d",
            "su", "si", "sd", "ui", "ud", "id",
            "sui", "sud", "siu", "sid", "sdu", "sdi",
            "uis", "uid", "usi", "usd", "udi", "uds",
            "ids", "idu", "dis", "diu",
            "suid", "sudi", "siud", "sidu", "sdui", "sdiu",
            "usid", "usdi", "uisd", "uids", "udsi", "udis",
            "isud", "isdu", "iusd", "iuds", "idus", "idsu",
            "dsui", "dsiu", "dusi", "duis", "disu", "dius"
        ]
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>()
    }



    fn extract_provider_from_line
    (
        &self,
        comp_line: &str
    ) -> String
    {
        let patterns =
        [
            "--provider=",
            "--bind-provider=",
        ];

        for word in comp_line.split_whitespace()
        {
            for pattern in &patterns
            {
                if let Some(rest) = word.strip_prefix(pattern)
                {
                    return rest.to_string();
                }
            }
        }

        self.read_provider()
    }



    fn extract_chat_from_line
    (
        &self,
        comp_line: &str
    )
    -> String
    {
        let patterns =
        [
            "--chat=",
            "--chat-provider=",
        ];

        for word in comp_line.split_whitespace()
        {
            for pattern in &patterns
            {
                if let Some(rest) = word.strip_prefix(pattern)
                {
                    return rest.to_string();
                }
            }
        }
        self.get_chat()
    }
}
