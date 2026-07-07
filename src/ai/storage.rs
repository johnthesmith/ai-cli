/*
    SPDX-License-Identifier: MIT
    SPDX-FileCopyrightText: 2026 Still Swamp
*/

/*
    Storage interface for CRUD operations on facts
    Each fact = one fact in text file

    facts are loaded into memory and saved on demand
*/

use std::fs;
use serde_json::json;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::cell::Ref;
use std::cell::RefMut;

use core::Log;
use core::State;

pub struct Storage
{
    /* Log element */
    pub log: Rc<RefCell<Log>>,
    /* Allow insert operations */
    allow_insert: bool,
    /* Allow delete operations */
    allow_delete: bool,
    /* Allow update operations */
    allow_update: bool,
    /* Storage state */
    state: State,
    /* In-memory facts loaded once  id -> actor,content */
    pub facts: BTreeMap
    <
        /* id */
        String,
        (
            /* origin = prompt|memory|history|prompt|clipboard|shell */
            String,
            /* action = read|add|remove|change */
            String,
            /* actor = tool|assistant|user */
            String,
            /* fact content */
            String
        )
    >,
    /* State */
    pub fact_delimiter: String
}



impl Storage
{
    /*
        Create new storage
    */
    pub fn new
    (
        /* Log structure */
        log: Rc<RefCell<Log>>
    )
    -> Self
    {
        Self
        {
            log: log,
            allow_insert: true,
            allow_delete: true,
            allow_update: true,
            state: State::ok(),

            facts: BTreeMap::new(),
            /* Default fact delimiter will be replaced by first promt line */
            fact_delimiter: "==fAcTd==".to_string()
        }
    }



    /*
        Parse facts from string content

        Format:
            delimiter
            id
            origin
            actor
            content
    */
    pub fn parse_file
    (
        &mut self,
        content: &str
    )
    {
        let lines: Vec<&str> = content.lines().collect();
        if !lines.is_empty()
        {
            let delimiter = lines[ 0 ].trim().to_string();
            self.fact_delimiter = delimiter.clone();

            if !self.fact_delimiter.is_empty()
            {
                self
                .get_log_mut()
                .trace( "Delimiter founded" )
                .prm( "value", delimiter );

                let mut facts = Vec::new();
                let mut current_fact = Vec::new();
                let mut i = 1; // skip first line (delimiter)

                while i < lines.len()
                {
                    let line = lines[i];
                    /*
                        Check if line is exactly the delimiter
                        (no extra chars)
                    */
                    if line == self.fact_delimiter
                    {
                        /* End of current fact */
                        if !current_fact.is_empty()
                        {
                            facts.push( current_fact.join( "\n" ));
                            current_fact.clear();
                        }
                    }
                    else
                    {
                        current_fact.push(line);
                    }

                    i += 1;
                }

                /* Last fact */
                if !current_fact.is_empty()
                {
                    facts.push( current_fact.join( "\n" ));
                }

                for fact in facts
                {
                    let fact = fact.trim();
                    if !fact.is_empty()
                    {
                        let lines: Vec<&str> = fact.lines().collect();
                        if lines.len() > 3
                        {
                            let id = lines[ 0 ].trim().to_string();
                            let origin = lines[ 1 ].trim().to_string();
                            let actor = lines[ 2 ].trim().to_string();
                            let content = lines[ 3.. ]
                            .join( "\n" )
                            .trim()
                            .to_string();

                            let final_id = if id == "-"
                            {
                                self.generate_id()
                            }
                            else
                            {
                                id
                            };

                            self.facts.insert
                            (
                                final_id,
                                (
                                    /* Origin */
                                    origin,
                                    /* Empty action */
                                    "".to_string(),
                                    actor,
                                    content
                                )
                            );
                        }
                    }
                }
            }
            else
            {
                self
                .get_log_mut()
                .warning( "Delimiter wasn't founded" );
            }
        }
    }



    /*
        Parse facts from LLM answer with actions string content
        Format
    */
    pub fn parse_answer
    (
        &mut self,
        content: &str
    )
    {
        let lines: Vec<&str> = content.lines().collect();
        if !lines.is_empty()
        {
            let delimiter = lines[ 0 ].trim().to_string();
            self.fact_delimiter = delimiter.clone();
            if !self.fact_delimiter.is_empty()
            {
                self
                .get_log_mut()
                .trace( "Delimiter founded" )
                .prm( "value", delimiter );

                let mut facts = Vec::new();
                let mut current_fact = Vec::new();
                let mut i = 1; // skip first line (delimiter)
                while i < lines.len()
                {
                    let line = lines[i];
                    /*
                        Check if line is exactly the delimiter (no extra chars)
                    */
                    if line == self.fact_delimiter
                    {
                        /* End of current fact */
                        if !current_fact.is_empty()
                        {
                            facts.push( current_fact.join( "\n" ));
                            current_fact.clear();
                        }
                    }
                    else
                    {
                        current_fact.push(line);
                    }

                    i += 1;
                }

                /* Last fact */
                if !current_fact.is_empty()
                {
                    facts.push(current_fact.join( "\n" ));
                }

                for fact in facts
                {
                    let fact = fact.trim();
                    if !fact.is_empty()
                    {
                        let lines: Vec<&str> = fact.lines().collect();
                        {
                            let directive = lines[0].trim().to_string();
                            let mut content = String::new();
                            let mut origin = String::new();
                            let mut action = String::new();
                            let mut actor = String::new();
                            let mut id = "-".to_string();
                            match directive.as_str()
                            {
                                "memory" |
                                "memory-add" |
                                "memory-insert" =>
                                {
                                    if lines.len() > 1
                                    {
                                        content = lines[ 1.. ]
                                        .join( "\n" )
                                        .trim().to_string();
                                        origin = "memory".to_string();
                                        action = "add".to_string();
                                        actor = "%assistant%".to_string();
                                    };

                                },

                                "prompt" |
                                "prompt-add" |
                                "prompt-insert" =>
                                {
                                    if lines.len() > 1
                                    {
                                        content = lines[ 1.. ]
                                        .join( "\n" )
                                        .trim()
                                        .to_string();

                                        origin = "prompt".to_string();
                                        action = "add".to_string();
                                        actor = "%assistant%".to_string();
                                    };
                                },

                                "shell" |
                                "shell-add" |
                                "shell-insert" =>
                                {
                                    if lines.len() > 1
                                    {
                                        content = lines[ 1.. ]
                                        .join( "\n" )
                                        .trim()
                                        .to_string();
                                        origin = "shell".to_string();
                                        action = "add".to_string();
                                        actor = "%assistant%".to_string();
                                    };
                                },

                                "pool" |
                                "pool-add" |
                                "pool-insert" =>
                                {
                                    if lines.len() > 1
                                    {
                                        content = lines[ 1.. ]
                                        .join( "\n" )
                                        .trim()
                                        .to_string();
                                        origin = "pool".to_string();
                                        action = "add".to_string();
                                        actor = crate::ai::ASSISTANT
                                        .to_string();
                                    };
                                },


                                "change" | "update"  =>
                                {
                                    if lines.len() > 2
                                    {
                                        action = "change".to_string();
                                        id = lines[ 1 ].to_string();
                                        content = lines[ 2.. ]
                                        .join( "\n" )
                                        .trim()
                                        .to_string();
                                    };
                                },

                                "remove" | "delete" =>
                                {
                                    if lines.len() > 1
                                    {
                                        action = "remove".to_string();
                                        id = lines[ 1 ].to_string();
                                    };
                                },

                                "buffer" |
                                "clipboard" |
                                "clipboard-add" =>
                                {
                                    if lines.len() >= 1
                                    {
                                        content = lines[ 1.. ]
                                        .join( "\n" )
                                        .trim()
                                        .to_string();
                                        origin = "clipboard".to_string();
                                        action = "add".to_string();
                                        actor = "%assistant%".to_string();
                                    };
                                },

                                "add" |
                                "insert" |
                                "history" |
                                "history-add" |
                                _ =>
                                {
                                    if lines.len() >= 1
                                    {
                                        content = lines[ 1.. ]
                                        .join( "\n" )
                                        .trim()
                                        .to_string();
                                        origin = "history".to_string();
                                        action = "add".to_string();
                                        actor = "%assistant%".to_string();
                                    };
                                },

                            }

                            id = if id == "-" { self.generate_id() } else { id };
                            self.facts.insert
                            (
                                id,
                                ( origin, action, actor, content )
                            );
                        }
                    }
                }
            }
            else
            {
                self
                .get_log_mut()
                .warning( "Delimiter wasn't founded" );
            }
        }
    }



    /*
        Load storage from file
        File format:
            delimiter line
            id line
            origin line
            action line
            actor line
            content line(s)
            ...
    */
    pub fn load
    (
        &mut self,
        path: &str
    )
    -> &mut Self
    {
        let content = match fs::read_to_string( path )
        {
            Ok( c ) => c,
            Err( e ) =>
            {
                if e.kind() == std::io::ErrorKind::NotFound
                {
                    return self;
                }
                self.state.set_state
                (
                    "storage-load-error", json!
                    (
                        {
                            "path": path,
                            "error": e.to_string()
                        }
                    )
                );
                return self;
            }
        };

        self.facts.clear();
        self.parse_file( &content );

        self
    }



    /*
        Save storage to file
        File format:
            delimiter
            id
            origin
            action
            actor
            content
            ...
    */
    pub fn save
    (
        &mut self,
        /* Path to storage file */
        path: &str
    )
    -> &mut Self
    {
        /* Ensure directory exists */
        if let Err( e ) = core::ensure_directory( path )
        {
            self.state.set_state
            (
                "storage-save-dir-error", json!
                (
                    {
                        "path": path,
                        "error": e.to_string()
                    }
                )
            );
            return self;
        }

        let content = self.to_request_string();

        if let Err(e) = fs::write( path, &content )
        {
            self.state.set_state
            (
                "storage-save-error", json!
                (
                    {
                        "path": path,
                        "error": e.to_string()
                    }
                )
            );
        }

        self
    }



    /*
        Generate new ID (microseconds timestamp)
    */
    fn generate_id( &self )
    -> String
    {
        use std::time::{ SystemTime, UNIX_EPOCH };

        let since_epoch = SystemTime::now()
        .duration_since( UNIX_EPOCH )
        .unwrap_or_default();

        since_epoch.as_micros().to_string()
    }



    /*
        Return state
    */
    pub fn get_state( &self )
    -> &State
    {
        &self.state
    }



    /*
        Return state mute
    */
    pub fn get_state_mut( &mut self )
    -> &mut State
    {
        &mut self.state
    }



    /*
        Clear all facts from storage
    */
    pub fn clear( &mut self )
    -> &mut Self
    {
        self.facts.clear();
        self
    }



    /**************************************************************************
        SIUD
    */


    /*
        Select fact by ID
        Returns (actor, content) or empty string if not found
    */
    #[allow(dead_code)]
    pub fn select
    (
        &mut self,
        /* Fact ID */
        id: &str
    )
    ->
    (
        /* Оrigin */
        String,
        /* Action */
        String,
        /* Actor */
        String,
        /* Сontent */
        String
    )
    {
        if let Some(( origin, action, actor, content )) = self.facts.get( id )
        {
            (
                origin.clone(),
                action.clone(),
                actor.clone(),
                content.clone()
            )
        }
        else
        {
            self.state.set_state
            (
                "storage-select-not-found",
                json!({ "id": id })
            );
            ( String::new(), String::new(), String::new(), String::new() )
        }
    }



    /*
        insert new fact
        Returns generated ID
    */
    pub fn insert
    (
        &mut self,
        /* Origin of fact */
        origin: &str,
        /* Action */
        action: &str,
        /* Actor */
        actor: &str,
        /* Content of fact */
        content: &str,
        /* True for disable check rights */
        no_right: bool
    )
    -> &mut Self
    {
        if self.get_state().is_ok()
        {
            if self.allow_insert || no_right
            {
                let id = self.generate_id();
                self.facts.insert
                (
                    id.clone(),
                    (
                        origin.to_string(),
                        action.to_string(),
                        actor.to_string(),
                        content.to_string()
                    )
                );
            }
            else
            {
                self.state.set_state
                (
                    "storage-insert-not-allowed",
                    json!
                    (
                        {
                            "origin": origin,
                            "action": action,
                            "actor": actor,
                            "content": content
                        }
                    )
                );
            }
        }
        self
    }



    /*
        Update existing fact
    */
    pub fn update
    (
        &mut self,
        /* Id */
        id: &str,
        /* Action */
        action: &str,
        /* Content of fact */
        content: &str,
        /* True for disable check rights */
        no_right: bool
    )
    -> &mut Self
    {
        if self.get_state().is_ok()
        {
            if self.allow_update || no_right
            {
                if let Some(( origin, _, actor, _ )) = self.facts.get( id )
                {
                    self.facts.insert
                    (
                        id.to_string(),
                        (
                            origin.to_string(),
                            action.to_string(),
                            actor.to_string(),
                            content.to_string()
                        )
                    );
                }
                else
                {
                    self.state.set_state
                    (
                        "fact-not-found-for-update",
                        json!({ "id": id })
                    );
                }
            }
            else
            {
                self.state.set_state
                (
                    "storage-update-not-allowed",
                    json!
                    (
                        {
                            "id": id,
                            "content": content
                        }
                    )
                );
            }
        }
        self
    }



    /*
        Delete fact by ID
    */
    pub fn delete
    (
        &mut self,
        id: &str,
        /* True for disable check rights */
        no_right: bool
    )
    -> &mut Self
    {
        if self.get_state().is_ok()
        {
            if self.allow_delete || no_right
            {
                self.facts.remove( id );
            }
            else
            {
                self.state.set_state
                (
                    "storage-delete-not-allowed",
                    json!({ "id": id })
                );
            }
        }
        self
    }


    /**************************************************************************

    */

    /*
        Get fact by ID as string with delimiter
    */
    pub fn to_string_by_id
    (
        &self,
        id: &str
    )
    -> String
    {
        if let Some(( origin, action, actor, content )) = self.facts.get( id )
        {
            return format!
            (
                "{}\n{}\n{}\n{}\n{}\n{}\n\n",
                self.fact_delimiter,
                id,
                origin,
                actor,
                action,
                content
            );
        }

        String::new()
    }



    /*
        Get fact by ID as string with delimiter
    */
    pub fn to_request_string_by_id
    (
        &self,
        id: &str
    )
    -> String
    {
        if let Some(( origin, _, actor, content )) = self.facts.get( id )
        {
            format!
            (
                "{}\n{}\n{}\n{}\n{}\n\n",
                self.fact_delimiter,
                id,
                origin,
                actor,
                content
            )
        }
        else
        {
            String::new()
        }
    }



    /*
        Get all facts as single string with delimiter
    */
    pub fn to_string
    (
        &self
    )
    -> String
    {
        let mut content = String::new();
        for( id, _ ) in &self.facts
        {
            content.push_str( &self.to_string_by_id( id ));
        }

        content
    }



    /*
        Get all facts as single string with delimiter
    */
    pub fn to_request_string
    (
        &self
    )
    -> String
    {
        let mut content = String::new();
        for( id, _ ) in &self.facts
        {
            content.push_str( &self.to_request_string_by_id( id ));
        }

        content
    }



    /*
    */
    pub fn set_access
    (
        &mut self,
        access: &str
    ) -> &mut Self
    {
        self.allow_insert = access.contains( 'i' );
        self.allow_update = access.contains( 'u' );
        self.allow_delete = access.contains( 'd' );
        self
    }



    pub fn get_access( &self )
    -> String
    {
        let mut access = String::new();
        if self.allow_insert { access.push( 'i' ); }
        if self.allow_update { access.push( 'u' ); }
        if self.allow_delete { access.push( 'd' ); }
        access
    }



    pub fn get_fact_delimiter( &self )
    -> String
    {
        self.fact_delimiter.clone()
    }



    pub fn set_fact_delimiter
    (
        &mut self,
        delimiter: &str
    )
    -> &mut Self
    {
        self.fact_delimiter = delimiter.to_string();
        self
    }



    pub fn exists
    (
        &mut self,
        id: &str
    ) -> bool
    {
        self.facts.contains_key(id)
    }



    /*
        Return shared reference to log
    */
    #[allow(dead_code)]
    pub fn get_log(&self)
    -> Ref<'_, Log>
    {
        self.log.borrow()
    }



    /*
        Return mutable reference to log (via RefCell)
    */
    #[allow(dead_code)]
    pub fn get_log_mut( &mut self )
    -> RefMut<'_, Log>
    {
        self.log.borrow_mut()
    }



    #[allow(dead_code)]
    pub fn get_log_rc(&self)
    -> Rc<RefCell<Log>>
    {
        self.log.clone()
    }
}
