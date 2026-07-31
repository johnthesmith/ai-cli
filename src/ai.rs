/*
    SPDX-License-Identifier: MIT
    SPDX-FileCopyrightText: 2026 Still Swamp
*/



/*
    Main AI module
*/

mod providers;
mod config;
mod help;
mod storage;
mod facts;


use serde_json::json;
use serde_json::Value as JsonValue;

use std::io::{ Read, Write, IsTerminal };
use core::{ App, SerdeExt, State, Moment, Color };
use storage::Storage;

use std::collections::BTreeMap;

pub const USER: &str = "user";
pub const ASSISTANT: &str = "assistant";
pub const AI_FOLDER: &str = ".ai-cli";



/*
    Ai applicatoin
*/
pub struct Ai
{
    /* Application structure */
    app: App,

    /* Profile for current session */
    profile: String,

    /* AI provider for current session */
    provider: String,

    /* Id of model of provider for current session */
    model: String,

    /* Chat id */
    chat: String,

    /* Memory id */
    memory_id: String,

    /* Id of current prompt */
    prompt_id: String,

    fact_delimiter: String,

    status: Vec<String>,

    storage: Storage,

    /* Access section */
    access_access: String,
    access_history: String,
    access_memory: String,
    access_prompt: String,
    access_shell: String,
    access_clipboard: String,
    access_read: String,
    access_write: String,

    no_prompt: bool,
    no_history: bool,
    no_memory: bool,

    /* id fact: file path */
    write_translation: BTreeMap < String, String >,

    /* Colorize */
    colorize: bool
}



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

            storage: Storage::new( log ),

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



    /*
        Return application
    */
    pub fn get_app( &self )
    -> &App
    {
        &self.app
    }



    /*
        Return application
    */
    #[allow(dead_code)]
    pub fn get_app_mut( &mut self )
    -> &mut App
    {
        &mut self.app
    }



    /*
        Help utility
    */
    fn help( &mut self )
    -> &mut Self
    {
        println!
        (
            "{}",
            help::CONTENT.replace( "%version%", &self.get_version() )
        );
        self
    }



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
                }
            }
        );

        println!( "{}", serde_yaml::to_string(&info).unwrap_or_default() );

        self
    }



    /*
        Init
    */
    fn init( &mut self )
    -> &mut Self
    {
        let path = self.get_profiles_path();
        if !path.is_empty()
        {
            self.app.state.set_state
            (
                "profile-already-exists",
                json!({ "path": path })
            );
        }
        else
        {
            let current = std::env::current_dir().unwrap();
            let ai_folder = current.join( AI_FOLDER );

            std::fs::create_dir_all(&ai_folder).unwrap();

            self.profile
            = current.file_name().unwrap().to_string_lossy().to_string();
        }

        self
    }



    /*
        Main run method
    */
    pub fn run( &mut self )
    -> &mut Self
    {
        /* Define completion mode */
        let completion_mode = std::env::var( "COMP_LINE" ).is_ok();

        let mut actions = Vec::new();

        /* Read cli arguments */
        self.app.read_cli();

        /* No prompt mode */
        let mut no_prompt = self.app.config[ "no-prompt" ].get_bool( false );

        /* No request mode */
        let mut no_request = self.app.config[ "no-request" ].get_bool( false );

        /* No errors */
        let mut no_error = false;

        if !completion_mode
        {
            /* Version request */
            if
            self.app.config[ "version" ].get_bool( false ) ||
            self.app.config[ "v" ].get_bool( false )
            {
                println!( "{}", self.get_version() );
                no_prompt = true;
                no_request = true;
                no_error = true;
            }

            /* Help request */
            if
            self.app.config[ "?" ].get_bool( false ) ||
            self.app.config[ "h" ].get_bool( false ) ||
            self.app.config[ "help" ].get_bool( false )
            {
                self.help();
                no_prompt = true;
                no_request = true;
                no_error = true;
            }

            /* Init flag */
            if self.app.config[ "init" ].get_bool( false )
            {
                self.init();
                no_prompt = true;
                no_request = true;
            }


            if self.app.config[ "tiocsti" ].get_bool( false )
            {
                /* Read from stdin */
                let mut input = String::new();
                match std::io::stdin().read_to_string( &mut input )
                {
                    Ok( 0 ) =>
                    {
                        self.app.state.set_state
                        (
                            "tiocsti-stdin-isempty",
                            json!({})
                        );
                    }
                    Ok( _ ) =>
                    {
                        self.input_tiocsti( &input );
                    }
                    Err( e ) =>
                    {
                        self.app.state.set_state
                        (
                            "tiocsti-failed-to-read-stdin",
                            json!({ "error": &e.to_string() })
                        );
                    }
                }
                no_prompt = true;
                no_request = true;
            }
        }

        /*
            Profile section
        */
        if self.app.state.is_ok()
        {
            if self.get_profiles_path().is_empty()
            {
                self.app.state.set_state
                (
                    "profile-not-found",
                    json!({ "reason": "Your need to run `ai --init`" })
                );
            }
        }

        if self.app.state.is_ok()
        {
            /* Let profile */
            let profile = self.app.config[ "bind-profile" ].get_str( "" );

            /* Bind profile */
            if !profile.is_empty()
            {
                self.write_profile( &profile );
                self.set_profile( &profile );
                no_prompt = true;
            }

            /* Read prfile */
            if !profile.is_empty()
            {
                /* Set profile for current session only */
                self.set_profile( &profile );
            }
            else
            {
                /* Read profile */
                self.read_profile();
            }
        }

        /*
            Config section
        */
        if self.app.state.is_ok()
        {
            /* Get default config path */
            let path = self.get_config_file();

            if !std::fs::metadata(&path).is_ok()
            {
                /* Generate config */
                self.generate_config();
            }

            if self.app.state.is_ok()
            {
                /* Read config */
                self
                .app
                .read_config( &path )
                .read_cli()
                .read_sets()
                .read_cli();
            }
        }

        /*
            Log section
        */
        if self.app.state.is_ok()
        {
            /* Set log file */
            if let Some( file ) = self.app.config
            [ "application" ][ "log" ][ "file" ].as_str()
            {
                let file = core::expand_path( file )
                .replace( "%profile-path%", &self.get_profile_path())
                .replace( "%profile%", &self.get_profile())
                ;
                self.app.get_log_mut().set_file_path( &file );
            }
        }


        /*
            config
        */
        if self.app.state.is_ok()
        {
            /* First log message */
            self.app.get_log_mut().begin
            (
                "=== Ai started ==============================================="
            );

            /*
                Main section
            */

            /* Dump configuration */
            self.app.dump_config();

            /* Collect actions from config into a map (copy values) */
            if let Some( mapping ) = self.app.config.as_object()
            {
                for( key, value ) in mapping
                {
                    let action = key.to_string();
                    let target = value.get_str( "" );
                    actions.push(( action, target ));
                }
            }

            /* Define chat */
            self.set_chat
            (
                &if let Some( chat )
                = self.app.config[ "chat" ].as_str()
                {
                    chat.to_string()
                }
                else if let Some( c )
                = self.app.config[ "c" ].as_str()
                {
                    c.to_string()
                }
                else
                {
                    self.read_chat()
                }
            );

            /* Define memory id */
            self.set_memory_id
            (
                &if let Some( memory )
                = self.app.config[ "memory" ].as_str()
                {
                    memory.to_string()
                }
                else if let Some( m ) = self.app.config[ "m" ].as_str()
                {
                    m.to_string()
                }
                else
                {
                    self.read_memory_of_chat()
                }
            );

            /* Define provider */
            self.set_provider
            (
                &if let Some( provider )
                = self.app.config[ "provider" ].as_str()
                {
                    provider.to_string()
                }
                else if let Some( p ) = self.app.config[ "p" ].as_str()
                {
                    p.to_string()
                }
                else
                {
                    self.read_provider()
                }
            );

            /* Define model */
            self.set_model
            (
                &if let Some( model )
                = self.app.config[ "model" ].as_str()
                {
                    model.to_string()
                }
                else if let Some( m ) = self.app.config[ "m" ].as_str()
                {
                    m.to_string()
                }
                else
                {
                    self.read_model()
                }
            );

            /* Define prompt */
            self.set_prompt_id
            (
                &if let Some( prompt )
                = self.app.config[ "prompt" ].as_str()
                {
                    prompt.to_string()
                }
                else if let Some( pt ) = self.app.config[ "pt" ].as_str()
                {
                    pt.to_string()
                }
                else
                {
                    self.read_prompt()
                }
            );

            let prompt_template
            = self.app.config[ "compile-prompt" ].get_str( "" );

            if !prompt_template.is_empty()
            {
                self.compile_prompt( &prompt_template );
            }

            /* Set no prompt flag */
            for( action, _ ) in &actions
            {
                ( no_prompt, no_request ) = match action.as_str()
                {
                    "i" | "info" => ( true, true ),

                    "rmh" => ( true, true ),
                    "reset-history" | "rh" => ( true, true ),
                    "reset-memory" | "rm" => ( true, true ),

                    "out-history" | "oh" => ( true, true ),
                    "out-memory" | "om" => ( true, true ),
                    "out-prompt" | "op" => ( false, true ),
                    "out-prompt-content" | "opc" => ( true, true ),
                    "tiocsti" => ( true, true ),
                    "completion" => ( true, true ),
                    "compile-prompt" => ( true, true ),
                    "bind-provider" => ( true, true ),
                    "bind-prompt" => ( true, true ),
                    "bind-model" => ( true, true ),
                    "bind-chat" => ( true, true ),
                    "bind-memory" => ( true, true ),
                    "sf" | "select-fact" => ( true, true ),
                    _ => ( no_prompt, no_request )
                };
            }

            /* Execute actions from collected map */
            for( action, target ) in &actions
            {
                match action.as_str()
                {
                    "bind-chat" =>
                    {
                        self.bind_chat( &target ).set_chat( &target );
                    }
                    "bind-provider" =>
                    {
                        self.bind_provider( &target ).set_provider( &target );
                    }
                    "bind-prompt" =>
                    {
                        self.bind_prompt( &target ).set_prompt_id( &target );
                    }
                    "bind-model" =>
                    {
                        self.bind_model( &target ).set_model( &target );
                    }
                    "bind-memory" =>
                    {
                        self.bind_memory( &target ).set_memory_id( &target );
                    }
                    "reset-history" | "rh" => { self.clear_history(); }
                    "reset-memory" | "rm"=> { self.clear_memory(); }
                    "rmh"=>
                    {
                        self.clear_memory().clear_history();
                    }
                    _ => {}
                }
            }

            /* Validate provider */
            if !self.provider_exists( &self.get_provider() )
            {
                self.app.state.set_state
                (
                    "unknown-provider",
                    json!
                    (
                        {
                            "requested-provider": &self.get_provider()
                        }
                    )
                );
            }
        }



        if self.app.state.is_ok() && !completion_mode
        {
            self.no_history = self.get_config_bool( &[ "no-history" ], false );
            self.no_memory = self.get_config_bool( &[ "no-memory" ], false );
            self.no_prompt = self.get_config_bool( &[ "no-prompt" ], false );

            /* Set access rights */
            self.access_access
            = self.get_config_str( &[ "access-access" ], "s" );
            self.access_history
            = self.get_config_str( &[ "access-history" ], "si" );
            self.access_memory
            = self.get_config_str( &[ "access-memory" ], "si" );
            self.access_prompt
            = self.get_config_str( &[ "access-prompt" ], "s" );
            self.access_shell
            = self.get_config_str( &[ "access-shell" ], "i" );
            self.access_clipboard
            = self.get_config_str( &[ "access-clipboard" ], "i" );
            self.access_read
            = self.get_config_str( &[ "access-read" ], "s" );
            self.access_write
            = self.get_config_str( &[ "access-write" ], "su" );

            /* Open prompt */
            self.ensure_prompt_file();

            /*
                Retrive user prompt
            */
            let( user_input, user_stdin ) = if no_prompt
            {
                (String::new(), String::new())
            }
            else
            {
                self.get_user_prompt()
            };

            /* Build prompt */
            self.build_prompt( &user_input, &user_stdin );

            if self.app.state.is_ok()
            {
                /* Processing with facts */
                for( action, target ) in &actions
                {
                    match action.as_str()
                    {
                        "delete-fact" =>
                        {
                            if !target.is_empty()
                            {
                                self.storage.delete( &target, true );
                            }
                        }

                        "insert-fact" =>
                        {
                            let actor
                            = self.app.config[ "actor" ].get_str( USER );

                            let domain
                            = self.app.config[ "domain" ].get_str( "history" );

                            let body = if target.is_empty()
                            {
                                let mut input = String::new();
                                std::io::stdin().read_to_string(&mut input).ok();
                                input.trim().to_string()
                            }
                            else
                            {
                                target.clone()
                            };

                            if !body.is_empty()
                            {
                                self.storage.insert
                                (
                                    "",
                                    &domain,
                                    &actor,
                                    &body,
                                    true
                                );
                             }
                        }

                        "update-fact" =>
                        {
                            let domain = self.app.config[ "domain" ]
                            .get_str( "history" );

                            let actor = self.app.config[ "actor" ]
                            .get_str( "read" );

                            let mut body = self.app.config[ "body" ]
                            .get_str( "" );

                            if !target.is_empty()
                            {
                                if body.is_empty()
                                {
                                    let mut input = String::new();
                                    std::io::stdin()
                                    .read_to_string(&mut input)
                                    .ok();
                                    body = input.trim().to_string();
                                }

                                if !body.is_empty()
                                {
                                    self.storage.update
                                    (
                                        &target,
                                        &domain,
                                        &actor,
                                        &body,
                                        true
                                    );
                                }
                            }
                        }
                        _ => {}
                    }
                }

                let prompt = self.storage.to_string();

                /* Execute actions from collected map */
                for( action, target ) in &actions
                {
                    match action.as_str()
                    {
                        "i" | "info" => { self.out_info(); }
                        "out-history" | "oh" => { self.out_history(); }
                        "out-memory" | "om" => { self.out_memory(); }
                        "out-prompt" | "op" => { println!( "{}", prompt ); }
                        "out-prompt-content" | "opc" => { self.out_prompt(); }
                        "sf" | "select-fact" =>
                        {
                            if !target.is_empty()
                            {
                                println!
                                (
                                    "{}",
                                    self.storage.to_string_by_id( &target )
                                );
                            }
                         }

                        _ => {}
                    }
                }


                /* Send chat request */
                if !no_request
                {
                    let size = prompt.len();
                    let max_bytes = self.get_max_prompt_bytes();

                    if size < max_bytes
                    {
                        let mut provider = providers::create_provider
                        (
                             &self.get_provider(),
                             self
                        );
                        provider.chat( &prompt );
                    }
                    else
                    {
                        println!
                        (
                            "Prompt size {} bytes exceeds limit {} bytes.\n\
                             Please increase max-prompt-bytes in config or cli",
                             size,
                             max_bytes
                        );
                    }
                }


                self.storage.split( &[ "history" ] ).save
                (
                    &self.get_history_file_path()
                );

                self.storage.split( &[ "memory" ] ).save
                (
                    &self.get_memory_file()
                );

                self.storage.split( &[ "prompt" ] ).save
                (
                    &self.get_prompt_content_file()
                );
            }
        }

        if completion_mode
        {
            self.completion();
        }
        else
        {
            /* Dump final state if its not ok*/
            if !self.app.state.is_ok()
            {
                if !no_error
                {
                    self.app.state.dump();
                }
            }
            else
            {
                /* Last log out */
                self.app.get_log_mut().end( "End of ai" ).eol();
            }
        }


        self
    }



    /***************************************************************************
        Prompt secion
    */

    /*
        Return file name with id of prompt
    */
    fn get_prompt_file
    (
        &self,
        chat_id: &str
    )
    -> String
    {
        core::expand_path
        (
            &self.get_config_str
            (
                &[ "prompt-file-id" ],
                "%chat-path%/prompt.txt"
            )
            .replace( "%chat-path%", &self.get_chat_path( chat_id ))
            .replace( "%profile%", &self.get_profile())
            .replace( "%chat%", &self.get_chat() )
            .replace( "%provider%", &self.get_provider() )
            .replace( "%model%", &self.get_model_safe())
        )
    }



    /*
        Return list of provider names
    */
    fn get_prompts_path
    (
        &self,
        chat_id: &str
    )
    -> String
    {
        core::expand_path
        (
            &self.get_config_str
            (
                &[ "prompts-path" ],
                "%chat-path%/prompts/"
            )
            .replace( "%chat-path%", &self.get_chat_path( chat_id ) )
            .replace( "%profile-path%", &self.get_profile_path() )
            .replace( "%profile%", &self.get_profile() )
            .replace( "%chat%", chat_id )
            .replace( "%provider%", &self.get_provider() )
            .replace( "%model%", &self.get_model_safe() )
        )
    }



    /*
        Return prompt file size
    */
    fn get_prompt_content_file_size( &self )
    -> u64
    {
        let path = self.get_prompt_content_file();
        std::fs::metadata( &path )
        .map( |m| m.len() )
        .unwrap_or( 0 )
    }



    /*
        Return list of prompts
    */
    fn get_prompts
    (
        &self,
        chat: &str
    )
    -> Vec<String>
    {
        let mut result = Vec::new();

        /* From config */
        if let Some(prompts) = self.app.config["prompts"].as_object()
        {
            for key in prompts.keys()
            {
                let name = key.to_string();
                if !result.contains(&name)
                {
                    result.push(name);
                }
            }
        }

        /* From flies */
        let path = self.get_prompts_path(chat);
        if let Ok(entries) = std::fs::read_dir(&path)
        {
            for entry in entries.flatten()
            {
                if let Ok(file_type) = entry.file_type()
                {
                    if file_type.is_file()
                    {
                        if let Some(name) = entry.file_name().to_str()
                        {
                            if name.ends_with(".txt")
                            {
                                let prompt_name = name.trim_end_matches(".txt").to_string();
                                if !result.contains(&prompt_name)
                                {
                                    result.push(prompt_name);
                                }
                            }
                        }
                    }
                }
            }
        }

        if result.is_empty()
        {
            result.push("default".to_string());
        }

        result
    }


    /*
        Return current prompt id
    */
    fn read_prompt( &self )
    -> String
    {
        let prompt_path = self.get_prompt_file( &self.get_chat() );
        match std::fs::read_to_string( &prompt_path )
        {
            Ok( content ) => content,
            Err(_) => "default".to_string()
        }
    }



    fn read_prompt_content( &mut self )
    -> String
    {
        let path = self.get_prompt_content_file();
        match std::fs::read_to_string( &path )
        {
            Ok(content) => content,
            Err(e) =>
            {
                self.app.get_log_mut()
                    .error("Failed to read prompt file")
                    .prm("path", &path)
                    .prm("error", &e.to_string());
                String::new()
            }
        }
    }




    fn out_prompt
    (
        &mut self
    )
    -> &mut Self
    {
        let content = self.read_prompt_content();
        if content.is_empty()
        {
            println!("No prompt");
        }
        else
        {
            println!("{}", content);
        }
        self
    }



    /*
        Compile prompt file in to current self.prompt
    */
    fn compile_prompt
    (
        &mut self,
        /* Prompt name */
        template: &str
    ) -> &mut Self
    {
        let config = &self.app.config;
        let facts = &config[ "prompts" ][ template ];

        if facts.is_null()
        {
            println!( "Template not found" );
        }
        else
        {
            let mut source = Storage::new( self.app.get_log_rc() );
            let mut dest = Storage::new( self.app.get_log_rc() );
            source.parse( facts::FACTS );

            if let Some( facts ) = facts.as_array()
            {
                for id in facts
                {
                    let id = id.get_str( "" );
                     if let Some(( domain, actor, content ))
                     = source.get_by_id( &id )
                     {
                         dest.insert( &id, domain, actor, &content, true );
                     }
                }

                let prompt_path = self.get_prompt_content_file();
                let _ = std::fs::write( &prompt_path, dest.to_string() );
            }
        }
        self
    }



    /*
        Write profile in to file
    */
    fn bind_prompt
    (
        &mut self,
        name: &str
    ) -> &mut Self
    {
        let path = self.get_prompt_file( &self.get_chat());

        if let Err(e) = std::fs::write( &path, name )
        {
            /* Set state for app */
            self.app.state.set_state
            (
                "prompt-write-error",
                json!
                (
                    {
                        "path": path,
                        "error": e.to_string()
                    }
                )
            );
            /* Write in to log */
            self.app.get_log_mut()
            .error( "Failed to write prompt id" )
            .prm( "path", &path )
            .prm( "error", &e.to_string() );
        }
        else
        {
            self.app.get_log_mut()
            .trace( "Prompt id saved" )
            .prm( "name", name );
        }

        self
    }



    /*
        Return chat for current session
    */
    fn get_prompt_id( &self )
    -> String
    {
        self.prompt_id.clone()
    }



    /*
        Set chat for current session
    */
    fn set_prompt_id
    (
        &mut self,
        id: &str
    )
    -> &mut Self
    {
        self.prompt_id = id.to_string();
        self
    }



    /*
        Return prompt file path for chat
    */
    fn get_prompt_content_file( &self )
    /* Return prompt file name */
    -> String
    {
        core::expand_path
        (
            &self.get_config_str
            (
                &[ "prompt-file" ],
                "%profile-path%/chats/%chat%/prompts/%prompt%.txt"
            )
            .replace( "%profile-path%", &self.get_profile_path() )
            .replace( "%profile%", &self.get_profile() )
            .replace( "%provider%", &self.get_provider() )
            .replace( "%model%", &self.get_model() )
            .replace( "%chat%", &self.get_chat() )
            .replace( "%prompt%", &self.get_prompt_id() )
        )
    }



    /*
       Return user prompt combining stdin pipe, CLI arguments,
       and interactive input.
    */
    fn get_user_prompt( &mut self )
    ->
    (
        /* cli input or interactive input */
        String,
        /* stdin */
        String
    )
    {
        let mut input = String::new();
        let mut stdin_data = String::new();
        let mut stdin = std::io::stdin();
        let is_pipe = !stdin.is_terminal();

        if is_pipe
        {
            match stdin.read_to_string(&mut stdin_data)
            {
                Ok(0) => {}

                Ok(_) => { stdin_data = stdin_data.trim().to_string(); }

                Err(e) =>
                {
                    self.app.get_log_mut()
                    .error("Failed to read from stdin pipe")
                    .prm("error", &e.to_string());
                }
            }
        }

        let args: Vec<String> = std::env::args().skip( 1 )
        .filter(|arg| !arg.starts_with( '-' ))
        .collect();

        if !args.is_empty()
        {
            input = args.join(" ");
        }

        if !is_pipe && input.is_empty() && stdin_data.is_empty()
        {
            println!
            (
                "Enter your prompt (Ctrl+D to finish or Ctrl+C to cancel):"
            );
            let mut interactive = String::new();
            if stdin.read_to_string(&mut interactive).unwrap_or(0) > 0
            {
                input = interactive.trim().to_string();
            }
            println!();
        }

        ( input, stdin_data )
    }



    /*
        Convert a compact access string ( e.g. "siud" ) into explicit
        words joined by "|" ( e.g. "select|insert|update|delete" ),
        since a bare letter code is not reliably understood by the LLM.
        An empty access string yields "none".
    */
    fn access_to_str
    (
        &self,
        /* Compact access string, e.g. "siud" */
        access: &str
    )
    -> String
    {
        let words: Vec<&str> = access
        .chars()
        .filter_map( |c|
            match c
            {
                's' => Some( "select" ),
                'i' => Some( "insert" ),
                'u' => Some( "update" ),
                'd' => Some( "delete" ),
                _ => None,
            }
        )
        .collect();

        if words.is_empty()
        {
            "none".to_string()
        }
        else
        {
            words.join( "|" )
        }
    }



    /*
        Insert files into the prompt storage
    */
    fn insert_files
    (
        &mut self,
        /* Type operation read|write */
        type_operation: &str
    )
    -> &mut Self
    {
        let files: Vec<String> = self.app.config[ type_operation ]
        .get_str( "" )
        .split( ',' )
        .filter( |s| !s.is_empty() )
        .map( |s| s.to_string() )
        .collect()
        ;

        for file in files
        {
            let content = if std::fs::metadata( &file ).is_ok()
            {
                std::fs::read_to_string( &file ).unwrap_or_default()
            }
            else
            {
                self.app.get_log_mut()
                .warning( "File not found" )
                .prm( "file", &file )
                .prm( "type", type_operation );
                String::new()
            };

            let id = file.clone();

            if type_operation == "write"
            {
                self.write_translation.insert( id.clone(), file.clone() );
            }

            self.storage.facts.insert
            (
                id.clone(),
                (
                    type_operation.to_string(),
                    "user".to_string(),
                    content
                )
            );

            self.app.get_log_mut()
            .info( "File added to prompt" )
            .prm( "file", &file )
            .prm( "type", &type_operation )
            .prm( "id", &id );
        }

        self
    }



    /*
        Build full prompt content from template and context
    */
    fn build_prompt
    (
        &mut self,
        user_input: &str,
        user_stdin: &str
    )
    -> &mut Self
    {
        /*
            Storage
        */
        let prompt_content = self.read_prompt_content();
        let memory_content = self.read_memory_content();
        let history_content = self.read_history_content();
        let access_content = self.build_access_fact_content();
        let env_content = self.build_env_fact_content();

        self.storage
        .parse( &prompt_content )
        .insert( "domain-permissions", "access", USER, &access_content, true )
        .insert( "user-env", "env", USER, &env_content, true );

        self.insert_files( "read" );
        self.insert_files( "write" );

        self.storage
        .parse( &memory_content )
        .parse( &history_content );

        /* Add user stdin as history */
        if !self.no_history && !user_stdin.is_empty()
        {
            /* Replace all stdin fact delimiter */
            let stdin = user_stdin.replace
            (
                &self.fact_delimiter,
                "`fact-delimiter`"
            );
            self.storage.insert( "", "history", USER, &stdin, true );
        }

        /* Add user input as history */
        if !self.no_history && !user_input.is_empty()
        {
            /* Replace all input fact delimiter */
            let input = user_input.replace
            (
                &self.fact_delimiter,
                "`fact-delimiter`"
            );
            self.storage.insert( "", "history", USER, &input, true );
        }
        self
    }



    fn build_env_fact_content( &self )
    -> String
    {
        r#"
This is current enviropment:
User shell: %shell%
Current chat: %chat%
Provider: %provider%
Your Model: %model-name%
Tool version: %version%
Now: %now%
"#
        .to_string()
        .replace( "%shell%", &self.get_config_str( &[ "shell" ], "/bin/bash" ))
        .replace( "%chat%", &self.get_chat() )
        .replace( "%provider%", &self.get_provider() )
        .replace( "%model-name%", &self.get_model_name() )
        .replace( "%version%", &self.get_version() )
        .replace
        (
            "%now%",
            &Moment::create().now().format( "%Y-%m-%d %H:%M:%S" )
        )
    }



    fn build_access_fact_content( &self )
    -> String
    {
        r#"
This is your accesses for domains:
access: %access-access%
history: %access-history%
memory: %access-memory%
prompt: %access-prompt%
read: %access-read%
write: %access-write%
clipboard: %access-clipboard%
shell: %access-shell%
"#
        .to_string()
        .replace
        (
            "%access-access%",
            &self.access_to_str( &self.access_access )
        )
        .replace
        (
            "%access-history%",
            &self.access_to_str( &self.access_history )
        )
        .replace
        (
            "%access-memory%",
            &self.access_to_str( &self.access_memory )
        )
        .replace
        (
            "%access-prompt%",
            &self.access_to_str( &self.access_prompt )
        )
        .replace
        (
            "%access-read%",
            &self.access_to_str( &self.access_read )
        )
        .replace
        (
            "%access-write%",
            &self.access_to_str( &self.access_write )
        )
        .replace
        (
            "%access-clipboard%",
            &self.access_to_str( &self.access_clipboard )
        )
        .replace
        (
            "%access-shell%",
            &self.access_to_str( &self.access_shell )
        )
    }



    /*
        Ensure prompt file
        - Check if prompt file exists
        - If not, create default prompt based on prompt
    */
    fn ensure_prompt_file( &mut self )
    -> &mut Self
    {
        let prompt_path = self.get_prompt_content_file();

        if
            std::fs::metadata( &prompt_path )
            .map(|m| m.len() == 0)
            .unwrap_or( true )
        {
            let prompt = self.get_prompt_id();
            self.compile_prompt( &prompt );
        }

        self
    }



    /**************************************************************************
        History section
    */

    fn get_history_file_path( &self )
    -> String
    {
        core::expand_path
        (
            &self.get_config_str
            (
                &[ "history" ],
                "%chat-path%/history.txt"
            )
            .replace( "%chat-path%", &self.get_chat_path( &self.get_chat() ))
            .replace( "%profile-path%", &self.get_profile_path() )
            .replace( "%profile%", &self.get_profile() )
            .replace( "%model%", &self.get_model_safe() )
            .replace( "%provider%", &self.get_provider() )
            .replace( "%chat%", &self.get_chat() )
        )
    }



    /*
        Return history file size
    */
    fn get_history_file_size( &self )
    -> u64
    {
        let path = self.get_history_file_path();
        std::fs::metadata(&path)
        .map(|m| m.len())
        .unwrap_or(0)
    }



    /*
        Clear history file
    */
    fn clear_history(&mut self) -> &mut Self
    {
        let history_path = self.get_history_file_path();
        if let Err( e ) = std::fs::remove_file( &history_path )
        {
            if e.kind() != std::io::ErrorKind::NotFound
            {
                self.app.get_log_mut()
                .error( "Failed to clear history" )
                .prm( "path", &history_path )
                .prm( "error", &e.to_string() );
            }
        }
        else
        {
            self.app.get_log_mut()
            .info( "History cleared" )
            .prm( "path", &history_path );
        }
        self
    }



    /*
        Read history content
    */
    fn read_history_content(&mut self)
    -> String
    {
        let path = self.get_history_file_path();

        match std::fs::read_to_string(&path)
        {
            Ok(content) => content,

            Err(e) if e.kind() == std::io::ErrorKind::NotFound =>
            {
                String::new()
            }

            Err(e) =>
            {
                self.app.get_log_mut()
                .error("Failed to read history file")
                .prm("path", &path)
                .prm("error", &e.to_string());

                String::new()
            }
        }
    }


    /*
        Send history to stdout
    */
    fn out_history(&mut self)
    -> &mut Self
    {
        let history = self.read_history_content();

        if history.is_empty()
        {
            println!("No history");
        }
        else
        {
            println!("{}", history);
        }

        self
    }



    /**************************************************************************
        Any
    */


    /*
        Return tool version
    */
    fn get_version( &self )
    -> String
    {
        format!( "AI CLI Utility v{}", env!( "CARGO_PKG_VERSION" ))
    }



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

        match std::fs::write(&path, config::DEFAULT.as_bytes())
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
        Return proxy for current provider
    */
    fn read_proxy( &self )
    -> String
    {
        self.get_config_str( &[ "proxy" ], &String::new() )
    }



    /**************************************************************************
        Token
    */

    /*
        Return token path for current provider
    */
    fn get_token_path( &self ) -> String
    {
        core::expand_path
        (
            &self.get_config_str
            (
                &[ "token" ],
                "~/.config/ai/app/cli/%profile%/tokens/%provider%.txt"
            )
            .replace( "%profile-path%", &self.get_profile_path() )
            .replace( "%profile%", &self.get_profile())
            .replace( "%chat%", &self.get_chat())
            .replace( "%provider%", &self.get_provider())
            .replace( "%model%", &self.get_model_safe())

        )
    }



    /*************************************************************************
        Model secion
    */

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
        .map(|obj| obj.keys().map(|k| k.to_string()).collect())
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



    /*************************************************************************
        Provider section
    */

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

        "github".to_string()
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



    /**************************************************************************
        Profile section
    */



    fn get_profiles_path( &self )
    -> String
    {
        let mut current = match std::env::current_dir()
        {
            Ok(dir) => dir,
            Err(_) => return String::new(),
        };
        loop
        {
            let ai_path = current.join( AI_FOLDER );
            if ai_path.exists() && ai_path.is_dir()
            {
                return ai_path.display().to_string();
            }
            if !current.pop()
            {
                break;
            }
        }
        String::new()
    }



    /*
        Return profile file
    */
    fn get_profile_file( &self )
    -> String
    {
        core::expand_path( &(self.get_profiles_path() + "/profile.txt" ))
    }




    fn get_profile_path( &self )
    -> String
    {
        core::expand_path
        (
            &(self.get_profiles_path() + "/profiles/" + &self.profile )
        )
    }



    /*
        Set profile
    */
    fn set_profile
    (
        &mut self,
        /* Profile name */
        name: &str
    )
    -> &mut Self
    {
        self.profile = name.to_string();
        self
    }



    /*
        Return profile
    */
    fn get_profile( &self )
    -> &str
    {
        &self.profile
    }



    /*
        Read and return profile
    */
    fn read_profile( &mut self )
    -> &mut Self
    {
        let path = self.get_profile_file();

        let profile = std::fs::read_to_string(&path)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "default".to_string());

        self.set_profile( &profile );
        self
    }



    /*
        Write profile in to file
    */
    fn write_profile
    (
        &mut self,
        name: &str
    ) -> &mut Self
    {
        let path = self.get_profile_file();

        if let Err(e) = std::fs::write( &path, name )
        {
            /* Set state for app */
            self.app.state.set_state
            (
                "PROFILE_WRITE_ERROR",
                json!
                (
                    {
                        "path": path,
                        "error": e.to_string()
                    }
                )
            );
            /* Write in to log */
            self.app.get_log_mut()
            .error( "Failed to write profile" )
            .prm( "path", &path)
            .prm( "error", &e.to_string());
        }
        else
        {
            self.app.get_log_mut()
            .trace( "Profile saved" )
            .prm( "name", name);
        }

        self
    }



    /*******************************************************************8******
        Chat section
    */



    /*
        Return file for current chat id
    */
    fn get_chat_file( &self )
    -> String
    {
        core::expand_path
        (
            &self.get_config_str
            (
                &[ "chat-file" ],
                "%profile-path%/chat.txt"
            )
            .replace( "%profile-path%", &self.get_profile_path())
            .replace( "%profile%", &self.get_profile() )
        )
    }



    fn get_chats_path( &self )
    -> String
    {
        core::expand_path
        (
            &self.get_config_str
            (
                &[ "chats-path" ],
                "%profile-path%/chats"
            )
            .replace( "%profile-path%", &self.get_profile_path())
        )
    }



    fn get_chat_path
    (
        &self,
        chat_id: &str
    )
    -> String
    {
        core::expand_path
        (
            &self.get_config_str
            (
                &[ "chat-path" ],
                "%chats-path%/%chat%"
            )
            .replace( "%profile-path%", &self.get_profile_path())
            .replace( "%chats-path%", &self.get_chats_path())
            .replace( "%chat%", chat_id )
        )
    }



    /*
        Return list of chats
    */
    fn get_chats( &self )
    -> Vec<String>
    {
        let mut result = Vec::new();
        result.push("default".to_string());
        let path = self.get_chats_path();
        if let Ok(entries) = std::fs::read_dir(&path)
        {
            for entry in entries.flatten()
            {
                if let Ok(file_type) = entry.file_type()
                {
                    if file_type.is_dir()
                    {
                        if let Some(name) = entry.file_name().to_str()
                        {
                            let chat_name = name.to_string();
                            if !result.contains(&chat_name)
                            {
                                result.push(chat_name);
                            }
                        }
                    }
                }
            }
        }
        result
    }



    /*
        Return max length for chat prompt
    */
    fn get_max_prompt_bytes( &self )
    -> usize
    {
        self.get_config_size( &[ "max-prompt-bytes" ], 100000 )
    }




    fn read_chat( &self )
    -> String
    {
        let path = self.get_chat_file();

        if let Ok( content ) = std::fs::read_to_string( &path )
        {
            let id = content.trim().to_string();
            if !id.is_empty()
            {
                return id;
            }
        }
        "default".to_string()
    }



    /*
        Change current chat id
    */
    fn bind_chat
    (
        &mut self,
        new_id: &str
    )
    -> &mut Self
    {
        let file_path = self.get_chat_file();

        if let Err(e) = core::ensure_directory( &file_path )
        {
            self.app.get_log_mut()
            .error( "Failed to ensure chat directory" )
            .prm( "error", &e);
            return self;
        }

        if let Err(e) = std::fs::write(&file_path, new_id)
        {
            self.app.get_log_mut()
            .error( "Failed to bind chat" )
            .prm( "path", &file_path)
            .prm( "error", &e.to_string());
        }
        else
        {
            self.app.get_log_mut()
            .trace( "Chat binded" )
            .prm( "id", new_id);
        }

        self
    }



    /*
        Return chat for current session
    */
    fn get_chat( &self )
    -> String
    {
        self.chat.clone()
    }



    /*
        Set chat for current session
    */
    fn set_chat
    (
        &mut self,
        name: &str
    )
    -> &mut Self
    {
        self.chat = name.to_string();
        self
    }



    /**************************************************************************
        Commands
    */

    /*
        Run destination command by identifier.
        Identifier: "command", "out"
    */
    fn run_destination
    (
        &mut self,
        data: &str,
        dest_type: &str,
        /* true for sync execute or false for async */
        wait: bool
    )
    {
        let command = self.get_config_str
        (
            &[ "destination", dest_type ],
            &String::new()
        );

        self.run_command( data, &command, wait );
    }



    /*
        Execute external command to insert the AI-generated text.
        Falls back to stdout if command execution fails.
    */
    fn run_command
    (
        &mut self,
        /* Data written to command's STDIN */
        data: &str,
        /* Command line for execution (passed to shell -c) */
        command: &str,
        /* true for sync execute or false for async */
        wait: bool
    )
    {
        if command.is_empty()
        {
            println!( "{}", data );
            return;
        }

        /* Retrive shell */
        let shell = self.get_config_str( &[ "shell" ], "/bin/bash" );

        /* Replace data in command */
        let data_arg = &data.replace( '"', "\"" );
        let run_command = &command.replace( "%data%", data_arg );

        match std::process::Command::new
        (
            shell
        )
        .arg( "-c" )
        .arg( run_command )
        .stdin( std::process::Stdio::piped() )
        .spawn()
        {
            Ok(mut child) =>
            {
                let data_len = data.len();

                if let Some( mut stdin ) = child.stdin.take()
                {
                    let _ = stdin.write_all(data.as_bytes());
                    let _ = stdin.flush();
                }

                if wait
                {
                    match child.wait()
                    {
                        Ok( exit_status ) =>
                        {
                            self.app.get_log_mut()
                            .info( "Command executed successfully" )
                            .prm( "command", command )
                            .prm( "data_bytes", data_len )
                            .prm
                            (
                                "exit_code",
                                exit_status.code().unwrap_or( -1 )
                            );
                        }
                        Err( e ) =>
                        {
                            self.app.get_log_mut()
                            .warning( "Failed to wait for command" )
                            .prm( "command", run_command)
                            .prm( "error", &e.to_string());
                        }
                    }
                }
                else
                {
                    self.app.get_log_mut()
                    .info( "Command spawned (no wait)" )
                    .prm( "command", run_command)
                    .prm( "data_bytes", data_len);

                    std::thread::spawn
                    (
                        move ||
                        {
                            let _ = child.wait();
                        }
                    );
                }
            }
            Err( e ) =>
            {
                self
                .app.get_log_mut()
                .error( "Failed to execute command" )
                .prm( "command", run_command )
                .prm( "data_bytes", data.len() )
                .prm( "error", &e.to_string() );
                println!( "{}", data );
            }
        }
    }



    /*
        Format text to maximum line width of N characters (not bytes)
        Splits at word boundaries when possible
    */
    #[allow(dead_code)]
    fn format_text(&self, text: &str, max_chars: usize) -> String
    {
        let mut result = String::new();
        let mut line = String::new();

        for word in text.split_whitespace()
        {
            // Check length in characters, not bytes
            let new_len = line.chars().count() + word.chars().count() + 1;

            if new_len > max_chars
            {
                if !line.is_empty()
                {
                    result.push_str(&line);
                    result.push('\n');
                    line.clear();
                }
                line.push_str(word);
            }
            else
            {
                if !line.is_empty()
                {
                    line.push(' ');
                }
                line.push_str(word);
            }
        }

        if !line.is_empty()
        {
            result.push_str(&line);
        }

        result
    }



    fn mnemo
    (
        &mut self,
        text: &str
    )
    {
        let mut color = Color::White;
        if text.contains( '+' )
        {
            color = Color::Cyan;
        }

        if text.contains( '^' )
        {
            color = Color::Magenta;
        }

        if text.contains( '-' )
        {
            color = Color::Yellow;
        }

        if text.contains( '!' )
        {
            color = Color::Red;
        }

        self.status.push
        (
            Color::colorize( color, text, Color::Default, self.colorize )
        );
    }



    /*
        Inject command directly into TTY using TIOCSTI.

        This makes the command appear in the user's terminal prompt as if
        typed. Does NOT press Enter - user can edit before executing.

        # Security Warning
        Requires `sudo sysctl -w dev.tty.legacy_tiocsti=1` on modern kernels.
        Disabled by default due to security risks. Only use in trusted
        environments.

        # Arguments
        * `cmd` - Command string to inject (without newline)
    */
    fn input_tiocsti
    (
        &mut self,
        cmd: &str
    )
    {
        // Clone the config value to avoid borrowing self
        let tty_device = self.get_config_str( &[ "tty_device" ], "/dev/tty" );

        match std::fs::OpenOptions::new().write( true ).open( &tty_device )
        {
            Ok(fd) =>
            {
                use std::os::unix::io::AsRawFd;
                let fd_raw = fd.as_raw_fd();
                for byte in cmd.bytes()
                {
                    let ret = unsafe
                    {
                        libc::ioctl(fd_raw, libc::TIOCSTI, &byte)
                    };
                    if ret != 0
                    {
                        self.app.get_log_mut()
                        .error( "TIOCSTI ioctl failed" )
                        .prm( "byte", &byte.to_string())
                        .prm
                        (
                            "error",
                             &std::io::Error::last_os_error().to_string()
                         );

                        break;
                    }
                }

                self.app.get_log_mut()
                .info( "Command injected via TIOCSTI" )
                .prm( "tty", &tty_device )
                .prm( "length", cmd.len() );
            }
            Err(e) =>
            {
                self.app.get_log_mut()
                .error( "Failed to open TTY device" )
                .prm( "device", &tty_device )
                .prm( "error", &e.to_string() );
                println!( "{}", cmd );
            }
        }
    }



    /**************************************************************************
        Memory section
    */


    /*
        Return memory id for current session
    */
    fn get_memory_id( &self )
    -> String
    {
        self.memory_id.clone()
    }



    /*
        Return memory id for current session
    */
    fn set_memory_id
    (
        &mut self,
        cmd: &str
    )
    -> &mut Self
    {
        self.memory_id = cmd.to_string();
        self
    }



    fn get_memory_path( &self )
    -> String
    {
        core::expand_path
        (
            &self.get_config_str
            (
                &[ "memory-path" ],
                "%profile-path%/memory"
            )
            .replace( "%profile-path%", &self.get_profile_path())
            .replace( "%chat%", &self.get_chat())
        )
    }



    /*
        Return memory file
    */
    fn get_memory_file( &self )
    -> String
    {
        core::expand_path
        (
            &self.get_config_str
            (
                &[ "memory-file" ],
                "%memory-path%/%memory-id%.txt"
            )
            .replace( "%memory-path%", &self.get_memory_path() )
            .replace( "%profile-path%", &self.get_profile_path() )
            .replace( "%profile%", &self.get_profile() )
            .replace( "%provider%", &self.get_provider() )
            .replace( "%model%", &self.get_model_safe() )
            .replace( "%memory-id%", &self.get_memory_id() )
            .replace( "%chat%", &self.get_chat() )
        )
    }



    /*
        Return list of memories
    */
    fn get_memories(&self) -> Vec<String>
    {
        let mut result = Vec::new();
        result.push("default".to_string());

        let path = self.get_memory_path();

        if let Ok(entries) = std::fs::read_dir(&path)
        {
            for entry in entries.flatten()
            {
                if let Ok(file_type) = entry.file_type()
                {
                    if file_type.is_file()
                    {
                        if let Some(name) = entry.file_name().to_str()
                        {
                            if name.ends_with(".txt")
                            {
                                let id = name
                                .trim_end_matches(".txt")
                                .to_string();
                                if !result.contains(&id)
                                {
                                    result.push(id);
                                }
                            }
                        }
                    }
                }
            }
        }
        result
    }



    /*
        Return memory of file for current chat
    */
    fn get_memory_of_chat_file( &self )
    -> String
    {
        core::expand_path
        (
            &self.get_config_str
            (
                &[ "memory-of-chat-file" ],
                "%profile-path%/chats/%chat%/memory.txt"
            )
            .replace( "%profile-path%", &self.get_profile_path() )
            .replace( "%profile%", &self.get_profile() )
            .replace( "%provider%", &self.get_provider() )
            .replace( "%model%", &self.get_model_safe() )
            .replace( "%memory-id%", &self.get_memory_id() )
            .replace( "%chat%", &self.get_chat() )
        )
    }



    fn read_memory_of_chat( &self )
    -> String
    {
        let path = self.get_memory_of_chat_file();

        if let Ok( content ) = std::fs::read_to_string( &path )
        {
            let id = content.trim().to_string();
            if !id.is_empty()
            {
                return id;
            }
        }

        "default".to_string()
    }



    /*
        Bind memory for current chat
    */
    fn bind_memory
    (
        &mut self,
        id: &str
    )
    -> &mut Self
    {
        let file_path = self.get_memory_of_chat_file();

        if let Err(e) = core::ensure_directory( &file_path )
        {
            self.app.get_log_mut()
            .error( "Failed to ensure memory directory" )
            .prm( "error", &e );
            return self;
        }

        if let Err(e) = std::fs::write( &file_path, id )
        {
            self.app.get_log_mut()
            .error( "Failed to bind memory" )
            .prm( "path", &file_path )
            .prm( "error", &e.to_string() );
        }
        else
        {
            self.app.get_log_mut()
            .trace( "Memory binded" )
            .prm( "id", id );
        }

        self
    }



    /*
        Return memory file size
    */
    fn get_memory_file_size( &self )
    -> u64
    {
        let path = self.get_memory_file();
        std::fs::metadata(&path)
        .map(|m| m.len())
        .unwrap_or(0)
    }



    /*
        Clear memory file for current chat
    */
    fn clear_memory( &mut self )
    -> &mut Self
    {
        let path = self.get_memory_file();

        match std::fs::remove_file(&path)
        {
            Ok(_) =>
            {
                self.app.get_log_mut()
                .info("Memory file removed")
                .prm("path", &path);
            }
            Err(e) =>
            {
                if e.kind() == std::io::ErrorKind::NotFound
                {
                    self.app.get_log_mut()
                    .info("Memory file already removed")
                    .prm("path", &path);
                }
                else
                {
                    self.app.get_log_mut()
                    .error("Failed to remove memory file")
                    .prm("path", &path)
                    .prm("error", &e.to_string());
                }
            }
        }
        self
    }



    fn read_memory_content( &mut self )
    -> String
    {
        let path = self.get_memory_file();
        match std::fs::read_to_string(&path)
        {
            Ok(content) => content,
            Err(e) =>
            {
                if e.kind() == std::io::ErrorKind::NotFound
                {
                    String::new()
                }
                else
                {
                    self.app.get_log_mut()
                        .error( "Failed to read memory file" )
                        .prm( "path", &path )
                        .prm( "error", &e.to_string() );
                    String::new()
                }
            }
        }
    }



    /*
        Send memory to stdout
    */
    fn out_memory( &mut self )
    -> &mut Self
    {
        let content = self.read_memory_content();
        if content.is_empty()
        {
            println!( "No memory" );
        }
        else
        {
            println!( "{}", content );
        }
        self
    }


    /**************************************************************************
        Providers methods
    */



    /*
        Processing chat response
    */
    pub fn handle_chat_response
    (
        &mut self,
        /* Content form llm */
        content: &str,
        /* Count of income tokens */
        tokens_in: u64,
        /* Count of outcome tokens */
        tokens_out: u64,
        tokens_billed: u64,
        tokens_cached: u64
    )
    {
        self.app.get_log_mut().dump( "LLM response", content );

        self.storage
        .set_access( "access", &self.access_access )
        .set_access( "history", &self.access_history )
        .set_access( "prompt", &self.access_prompt )
        .set_access( "memory", &self.access_memory )
        .set_access( "shell", &self.access_shell )
        .set_access( "clipboard", &self.access_clipboard )
        .set_access( "read", &self.access_read )
        .set_access( "write", &self.access_write )
        ;

        /* Dump table */
        self.app.get_log_mut()
        .info( "Write file mapping" )
        .prm( "mapping", &format!( "{:?}", self.write_translation ));

        /* Answer is parsing */
        let mut response_storage = Storage::new( self.app.get_log_rc() );

        response_storage
        .set_fact_delimiter( &self.fact_delimiter )
        .parse( content );

        if !response_storage.facts.is_empty()
        {
            for( id, ( domain, actor, body )) in response_storage.facts.iter()
            {
                match domain.as_str()
                {
                    "read" =>
                    {
                        self.mnemo( "^!r" );
                    }

                    "write" =>
                    {
                        if Ai::check_access( &self.access_write, "u" )
                        {
                            let file_path = self
                            .write_translation
                            .get( id.as_str() )
                            .cloned();

                            if let Some( file_path ) = file_path
                            {
                                if let Err( e )
                                = core::ensure_directory(&file_path)
                                {
                                    self.mnemo("^!w");
                                    self.app.get_log_mut()
                                    .error("Failed to create directory")
                                    .prm("path", &file_path)
                                    .prm("error", &e);
                                }
                                else
                                {
                                    match std::fs::write( &file_path, &body )
                                    {
                                        Ok(_) =>
                                        {
                                            self.mnemo( "^w" );
                                            self.app.get_log_mut()
                                            .info( "Write file updated" )
                                            .prm( "path", file_path )
                                            .prm( "id", &id )
                                            .prm( "size", body.len() )
                                            ;
                                        }
                                        Err(e) =>
                                        {
                                            self.mnemo( "^!w" );
                                            self.app.get_log_mut()
                                            .error( "Failed to write file" )
                                            .prm( "path", file_path )
                                            .prm( "id", &id )
                                            .prm( "error", &e.to_string() );
                                        }
                                    }
                                }
                            }
                            else
                            {
                                self.mnemo( "^!w" );
                                self.app.get_log_mut()
                                .error( "Write file not found in translation" )
                                .prm( "id", &id );
                            }
                        }
                        else
                        {
                            self.mnemo( "^!w" );
                            self.app.get_log_mut()
                                .warning( "Write not allowed" )
                                .prm( "id", &id );
                        }
                    }

                    "clipboard" =>
                    {
                        if Ai::check_access( &self.access_clipboard, "i" )
                        {
                            self.run_destination( &body, "clipboard", true );
                            self.mnemo( "+c" );
                        }
                        else
                        {
                            self.run_destination
                            (
                               &Color::colorize
                               (
                                    Color::Yellow,
                                    &body,
                                    Color::Default,
                                    self.colorize
                                ),
                                "message",
                                true
                            );
                            self.mnemo( "+!c" );
                        }
                    }

                    /* Execute comma1nd via destination */
                    "shell" =>
                    {
                        if
                            self.app.config[ "no-shell" ].get_bool( false ) ||
                            self.app.config[ "nsh" ].get_bool( false )
                        {
                            self.run_destination
                            (
                               &Color::colorize
                               (
                                    Color::Yellow,
                                    &body,
                                    Color::Default,
                                    self.colorize
                                ),
                                "message",
                                true
                            );
                        }
                        else
                        {
                            if !self.no_history
                            {
                                self.storage.insert
                                (
                                    "",
                                    "history",
                                    ASSISTANT,
                                    body,
                                    false
                                );
                            }

                            /* Check if command execution is disabled */
                            if self.app.config[ "no-shell" ].get_bool( false )
                            {
                                self.app.get_log_mut()
                                .info
                                (
                                    "Command execution disabled by --no-shell"
                                )
                                .prm( "exec", &body );
                            }
                            else
                            {
                                /*
                                    REMOVE_ENTER

                                    Removes newline and carriage return
                                    characters from LLM-generated command.
                                    Prevents command injection via line breaks
                                    that could:
                                    1. Terminate the current command
                                    2. Inject arbitrary new commands
                                    3. Execute hidden malicious code

                                    The cleaned command remains as a single
                                    line. Only newline/carriage return are
                                    removed all other characters (&&, |, ;, $,
                                    `, etc.) are preserved as legitimate
                                    command syntax.
                                */
                                let clean_command = body
                                .replace( '\n', " " )
                                .replace( '\r', "" )
                                ;

                                self.run_destination
                                (
                                    &clean_command,
                                    "command",
                                    false
                                );

                                if body.contains( '\n' )
                                {
                                    self.mnemo( "+!s" );
                                }
                                else
                                {
                                    self.mnemo( "+s" );
                                };
                            }
                        }
                    }

                    /* Handle memory operations */
                    "memory" =>
                    {
                        self.storage.get_state_mut().set_ok();

                        if self.no_memory
                        {
                            println!( "{}", &body );
                        }
                        else
                        {
                            let exists = self.storage.exists( &id );
                            if body.is_empty()
                            {
                                if exists
                                {
                                    self.storage.delete( &id, false );
                                    if self.storage.get_state().is_ok()
                                    {
                                        self.mnemo( "-m" );
                                        self.app.get_log_mut()
                                        .info( "Memory fact delited" )
                                        .prm( "text", &body );
                                    }
                                    else
                                    {
                                        self.mnemo( "-!m" );
                                        self.app.get_log_mut()
                                        .warning( "Memory fact wasn't delete" )
                                        .prm( "text", &body )
                                        .dump_state( self.storage.get_state() )
                                        ;
                                    }
                                }
                            }
                            else
                            {
                                if exists
                                {
                                    self.storage.update
                                    (
                                        &id,
                                        &domain,
                                        &actor,
                                        &body,
                                        false
                                    );
                                    if self.storage.get_state().is_ok()
                                    {
                                        self.mnemo( "^m" );
                                        self.app.get_log_mut()
                                        .info( "Memory fact updated" )
                                        .prm( "text", &body );
                                    }
                                    else
                                    {
                                        self.mnemo( "^!m" );
                                        self.app.get_log_mut()
                                        .warning( "Memory fact wasn't updated" )
                                        .prm( "text", &body )
                                        .dump_state( self.storage.get_state() )
                                        ;
                                    }
                                }
                                else
                                {
                                    self.storage.insert
                                    (
                                        "",
                                        &domain,
                                        &actor,
                                        &body,
                                        false
                                    );
                                    if self.storage.get_state().is_ok()
                                    {
                                        self.mnemo( "+m" );
                                        self.app.get_log_mut()
                                        .info( "Memory fact added" )
                                        .prm( "text", &body );
                                    }
                                    else
                                    {
                                        self.mnemo( "+!m" );
                                        self.app.get_log_mut()
                                        .warning( "Memory fact wasn't added" )
                                        .prm( "text", &body )
                                        .dump_state( self.storage.get_state() )
                                        ;
                                    }
                                }
                            }
                        }
                    }

                    /* Handle history operations */
                    "history" =>
                    {
                        self.storage.get_state_mut().set_ok();
                        if self.no_history
                        {
                            println!( "{}", &body );
                        }
                        else
                        {
                            let exists = self.storage.exists( &id );
                            if body.is_empty()
                            {
                                if exists
                                {
                                    self.storage.delete( &id, false );
                                    if self.storage.get_state().is_ok()
                                    {
                                        self.mnemo( "-h" );
                                        self.app.get_log_mut()
                                        .info( "History fact delited" )
                                        .prm( "text", &body );
                                    }
                                    else
                                    {
                                        self.mnemo( "-!h" );
                                        self.app.get_log_mut()
                                        .warning( "History fact wasn't delite" )
                                        .prm( "text", &body )
                                        .dump_state( self.storage.get_state() )
                                        ;
                                    }
                                }
                            }
                            else
                            {
                                self.run_destination
                                (
                                   &Color::colorize
                                   (
                                        Color::Cyan,
                                        &body,
                                        Color::Default,
                                        self.colorize
                                    ),
                                    "message",
                                    true
                                );

                                if exists
                                {
                                    self.storage.update
                                    (
                                        &id,
                                        &domain,
                                        &actor,
                                        &body,
                                        false
                                    );

                                    if self.storage.get_state().is_ok()
                                    {
                                        self.mnemo( "^h" );
                                        self.app.get_log_mut()
                                        .info( "History fact updated" )
                                        .prm( "text", &body );
                                    }
                                    else
                                    {
                                        self.mnemo( "^!h" );
                                        self.app.get_log_mut()
                                        .warning
                                        (
                                            "History fact wasn't updated"
                                        )
                                        .prm( "text", &body )
                                        .dump_state
                                        (
                                            self.storage.get_state()
                                        );
                                    }
                                }
                                else
                                {
                                    self.storage.insert
                                    (
                                        "",
                                        &domain,
                                        &actor,
                                        &body,
                                        false
                                    );

                                    if self.storage.get_state().is_ok()
                                    {
                                        self.mnemo( "+h" );
                                        self.app.get_log_mut()
                                        .info( "History fact added" )
                                        .prm( "text", &body );
                                    }
                                    else
                                    {
                                        self.mnemo( "+!h" );
                                        self.app.get_log_mut()
                                        .warning( "History fact wasn't added" )
                                        .prm( "text", &body )
                                        .dump_state
                                        (
                                            self.storage.get_state()
                                        );
                                    }
                                }
                            }
                        }
                    }

                    /* Handle prompt operations */
                    "prompt" =>
                    {
                        self.storage.get_state_mut().set_ok();

                        if self.no_prompt
                        {
                            println!( "{}", &body );
                        }
                        else
                        {
                            let exists = self.storage.exists( &id );

                            if body.is_empty()
                            {
                                if exists
                                {
                                    self.storage.delete( &id, false );
                                    if self.storage.get_state().is_ok()
                                    {
                                        self.mnemo( "-p" );
                                        self.app.get_log_mut()
                                        .info( "Prompt fact delited" )
                                        .prm( "text", &body );
                                    }
                                    else
                                    {
                                        self.mnemo( "-!p" );
                                        self.app.get_log_mut()
                                        .warning( "Prompt fact wasn't delite" )
                                        .prm( "text", &body )
                                        .dump_state
                                        (
                                            self.storage.get_state()
                                        );
                                    }
                                }
                            }
                            else
                            {
                                self.run_destination( &body, "message", true );

                                if exists
                                {
                                    self.storage.update
                                    (
                                        &id,
                                        &domain,
                                        &actor,
                                        &body,
                                        false
                                    );

                                    if self.storage.get_state().is_ok()
                                    {
                                        self.mnemo( "^p" );
                                        self.app.get_log_mut()
                                        .info( "Prompt fact updated" )
                                        .prm( "text", &body );
                                    }
                                    else
                                    {
                                        self.mnemo( "^!p" );
                                        self.app.get_log_mut()
                                        .warning( "Prompt fact wasn't updated" )
                                        .prm( "text", &body )
                                        .dump_state
                                        (
                                            self.storage.get_state()
                                        );
                                    }
                                }
                                else
                                {
                                    self.storage.insert
                                    (
                                        "",
                                        &domain,
                                        &actor,
                                        &body,
                                        false
                                    );

                                    if self.storage.get_state().is_ok()
                                    {
                                        self.mnemo( "+p" );
                                        self.app.get_log_mut()
                                        .info( "Prompt fact added" )
                                        .prm( "text", &body );
                                    }
                                    else
                                    {
                                        self.mnemo( "+!p" );
                                        self.app.get_log_mut()
                                        .warning( "Prompt fact wasn't added" )
                                        .prm( "text", &body )
                                        .dump_state
                                        (
                                            self.storage.get_state()
                                        );
                                    }
                                }
                            }
                        }
                    }

                    /* */
                    _ =>
                    {
                        self.app.get_log_mut()
                        .warning( "Formt error" )
                        .prm( "domain", domain )
                        .prm( "actor", actor )
                        .prm( "id", id )
                        .prm( "body", &body );
                        self.mnemo( "!!" );

                        let body = format!
                        (
                             "Unrecognized fact:\n{}\n{}\n{}\n{}\n",
                             domain,
                             actor,
                             id,
                             body
                        );

                        if !self.no_history
                        {
                            self.storage.insert
                            (
                                "",
                                "history",
                                ASSISTANT,
                                &body,
                                false
                            );
                        }

                        self.run_destination
                        (
                            &body,
                            "message",
                            true
                        );
                    }
                }
            }
        }
        else
        {
            self.run_destination( &content, "message", true );
            self.mnemo( "!E" );
        }

        if self.get_config_bool( &[ "out-status" ], false )
        {
            let full_status = self.status.join
            (
                &Color::colorize
                (
                    Color::Gray,
                    "|",
                    Color::Default,
                    self.colorize
                )
            );
            println!
            (
                "{}  byte h:{} | m:{} | p:{}   tokens in:{} | out:{} | bill:{} | cache:{}",
                full_status,
                self.get_history_file_size(),
                self.get_memory_file_size(),
                self.get_prompt_content_file_size(),
                tokens_in.to_string(),
                tokens_out.to_string(),
                tokens_billed.to_string(),
                tokens_cached.to_string()
            );
        }
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
        /* Provider name (e.g., "github", "openai", "deepseek" ) */
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
        Check right
    */
    fn check_access
    (
        rights: &str,
        right: &str
    )
    -> bool
    {
        rights.contains( right )
    }




    /**************************************************************************
        Completion
    */

    /*
        Bash completion handler
    */
    pub fn completion( &mut self )
    {
        /* Get default config path */
        let path = self.get_config_file();

        if !std::fs::metadata( &path ).is_ok()
        {
            /* Generate config */
            self.generate_config();
        }

        if self.app.state.is_ok()
        {
            /* Read config */
            self
            .app
            .read_config( &path )
            .read_cli()
            .read_sets()
            .read_cli();
        }

        let comp_line = std::env::var( "COMP_LINE" ).unwrap_or_default();
        let point_str = std::env::var( "COMP_POINT" ).unwrap_or_default();
        let comp_point: usize = point_str.parse().unwrap_or( 0 );

        /* Find safe character boundary */
        let end =
        if
            comp_point <= comp_line.len()
            && comp_line.is_char_boundary( comp_point )
        {
            comp_point
        }
        else
        {
            let mut end = comp_point;
            while end > 0 && !comp_line.is_char_boundary( end )
            {
                end -= 1;
            }
            end
        };

        let prefix = &comp_line[ ..end ];
        let parts: Vec<&str> = prefix.split( ' ' ).collect();
        let current = *parts.last().unwrap_or(&"");

        /* Extract key value from current part */
        let ( key, val ) = if let Some( eq_pos ) = current.find( '=' )
        {
            ( &current[..eq_pos], &current[eq_pos + 1..] )
        }
        else
        {
            ( current, "")
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
                if key == keys[ 0 ]
                {
                    /* Key complete and need values */
                    let values = self.find_values( key, val, &comp_line );
                    match values.len()
                    {
                        0 => {}
                        1 =>
                        {
                            list.push( format!( "{}", values[ 0 ] ))
                        }
                        _ =>
                        {
                            /* All values dump */
                            list = values.clone();
                        }
                    }
                }
                else
                {
                    /* Key funded but uncompleted */
                    if self.is_key_argument( &keys[ 0 ])
                    {
                        /* One key with  = */
                        list.push( format!( "{}=", keys[ 0 ] ))
                    }
                    else
                    {
                        /* One key without equal */
                        list.push( keys[ 0 ].clone() )
                    }
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
        Return true if key exists
    */
    fn is_key_argument
    (
        &mut self,
        key: &str
    )
    -> bool
    {
        vec![
            "--set",

            /* Profile & provider & model & chat (temporary) */
            "--profile",
            "--provider",
            "--model",
            "--memory",
            "--chat",
            "--prompt",

            /* Files */
            "--read",
            "--write",

            /* Permanent bind */
            "--bind-profile",
            "--bind-provider",
            "--bind-prompt",
            "--bind-model",
            "--bind-chat",
            "--bind-memory",

            /* LLM access rights (iud) */
            "--access-history",
            "--access-memory",
            "--access-prompt",

            /* Storage operations: history */
            "--select-history",
            "--delete-history",
            "--update-history",
            "--insert-history",

            /* Storage operations: memory */
            "--select-memory",
            "--delete-memory",
            "--update-memory",
            "--insert-memory",

            "--compile-prompt",

            /* Actor & body for insert/update */
            "--actor",
            "--body"
        ]
        .iter().any( |k| k == &key )
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
            "-i",
            "--info",
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
                /* Session control */
                "--no-prompt",
                "--no-command",

                "--set",

                /* Files */
                "--read",
                "--write",

                /* Profile & provider & model & chat (temporary) */
                "--profile",
                "--provider",
                "-m",
                "--model",
                "-c",
                "--chat",
                "--memory",
                "-p",
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

                /* Storage operations: history */
                "-oh",
                "--out-history",
                "-rh",
                "--remove-history",
                "--select-history",
                "--delete-history",
                "--update-history",
                "--insert-history",

                // Storage operations: memory
                "-om",
                "--out-memory",
                "-rm",
                "--remove-memory",
                "--select-memory",
                "--delete-memory",
                "--update-memory",
                "--insert-memory",

                // Prompt
                "-op",
                "--out-prompt",
                "-opc",
                "--out-prompt-content",
                "--compile-prompt",

                // Actor & body for insert/update
                "--actor",
                "--body",

                // Specific features
                "--tiocsti",
            ].iter().for_each(|k| result.push( k.to_string() ));
        }

        result
        .iter()
        .filter( |k| k.starts_with( key ))
        .map( |k| k.to_string())
        .collect()
    }



    /*
        Returns a list of possible values for a given key.
        Filters values by the provided val prefix.
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
        let value_map =
        [
            ( "--read", self.get_files( val )),
            ( "--write", self.get_files( val )),

            ( "--set", self.app.get_sets()),

            ( "--chat", self.get_chats()),
            ( "--bind-chat", self.get_chats()),

            ( "-p", self.get_providers( )),
            ( "--provider", self.get_providers( )),
            ( "--bind-provider", self.get_providers()),

            (
                "--model",
                self.get_models( &self.extract_provider_from_line( comp_line ))
            ),

            (
                "--bind-model",
                self.get_models( &self.extract_provider_from_line( comp_line ))
            ),

            ( "--memory", self.get_memories()),

            ( "--bind-memory", self.get_memories()),

            (
                "--prompt",
                self.get_prompts( &self.extract_chat_from_line( comp_line ))
            ),

            (
                "--bind-prompt",
                self.get_prompts( &self.extract_chat_from_line( comp_line ))
            ),

            (
                "--compile-prompt",
                self.get_prompts( &self.extract_chat_from_line( comp_line ))
            ),

            ( "--access-history", self.get_complete_value_access()),
            ( "--access-memory", self.get_complete_value_access()),
            ( "--access-prompt", self.get_complete_value_access()),
        ];

        for (k, values) in value_map.iter()
        {
            if *k == key
            {
                return values
                    .iter()
                    .filter(|v| v.starts_with(val))
                    .map(|v| v.to_string())
                    .collect();
            }
        }

        vec![format!("{} ", key)]
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
            "-p=",
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
    ) -> String
    {
        let patterns =
        [
            "--chat=",
            "-c=",
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


    /*
        Return list of files matching the path prefix
    */

    pub fn get_files(&self, val: &str)
    -> Vec<String>
    {
        let mut result = Vec::new();
        let path = std::path::Path::new(val);

        let dir = if val.ends_with('/')
        {
            path.to_path_buf()
        }
        else if let Some(parent) = path.parent()
        {
            parent.to_path_buf()
        }
        else
        {
            std::path::PathBuf::from(".")
        };

        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    let name = entry.file_name().to_string_lossy().to_string();
                        let full_path = dir.join(&name);
                        let display = full_path.to_string_lossy().to_string();
                        if file_type.is_dir() {
                            result.push(format!("{}/", display));
                        } else {
                            result.push(display);
                        }
                }
            }
        }

        result
    }





}



