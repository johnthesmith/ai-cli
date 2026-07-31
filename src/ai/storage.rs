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
use indexmap::IndexMap;
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

    last_generated_id: u128,

    /* Storage state */
    state: State,

    /* In-memory facts loaded once  id -> actor,content */
    pub facts: IndexMap
    <
        /* id */
        String,
        (
            /* domain = prompt|memory|history|pool|clipboard|shell */
            String,
            /* actor = assistant|user */
            String,
            /* fact content */
            String
        )
    >,

    pub access: BTreeMap
    <
        /* domain */
        String,
        /* SIUD */
        String
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

            last_generated_id: 0,

            state: State::ok(),

            facts: IndexMap::new(),
            access: BTreeMap::new(),

            /* Default fact delimiter will be replaced by first promt line */
            fact_delimiter: "#FACT".to_string()
        }
    }



    pub fn parse
    (
        &mut self,
        content: &str
    )
    -> &mut Self
    {
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        let mut current_content = Vec::new();
        let mut current_header: Option<(String, String, String)> = None;

        while i < lines.len()
        {
            let line = lines[ i ];

            if let Some(( domain, actor, id, _ )) = self.parse_header( line )
            {
                if let Some(( domain, actor, id)) = current_header.take()
                {
                    let content = current_content
                    .join( "\n" )
                    .trim()
                    .to_string();

                    let final_id = if id == "-" || id == "NEW" || id == "new"
                    {
                        self.generate_id()
                    }
                    else
                    {
                        id
                    };
                    self.facts.insert( final_id, (domain, actor, content ));
                    current_content.clear();
                }
                current_header = Some((domain, actor, id));
            }
            else
            {
                if current_header.is_some()
                {
                    current_content.push(line);
                }
            }
            i += 1;
        }

        if let Some((domain, actor, id)) = current_header.take()
        {
            let content = current_content
            .join( "\n" )
            .trim()
            .to_string();

            let final_id = if id == "-" || id == "NEW" || id == "new"
            {
                self.generate_id()
            }
            else
            {
                id
            };
            self.facts.insert( final_id, ( domain, actor, content ));
        }
        self
    }



    fn parse_header
    (
        &self, line: &str
    )
    -> Option
    <(
        /* domain */
        String,
        /* actor */
        String,
        /* id */
        String,
        /* content */
        String
    )>
    {
        let parts: Vec<&str> = line.split( '|' ).collect();

        if parts.len() >= 2 && parts[0] == "#FACT"
        {
            let domain = parts[ 1 ].to_string();

            let actor = if parts.len() >=3
            { parts[ 2 ].to_string() } else { String::new() };

            let id = if parts.len() >= 4
            { parts[ 3 ].to_string() } else { String::new() };

            let content = if parts.len() >= 5
            { parts[ 4.. ].join("|") } else { String::new() };

            return Some(( domain, actor, id, content ));
        }
        None
    }


    #[allow(dead_code)]
    pub fn clear_empty_content
    (
        &mut self
    )
    -> &mut Self
    {
        self.facts.retain
        (
            |_, (_, _, content)| !content.trim().is_empty()
        );
        self
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
    #[allow(dead_code)]
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
        self.parse( &content );

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

        let content = self.to_string();

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
        Create new storage containing only selected domains
    */
    pub fn split
    (
        &self,
        /* Domains to include */
        domains: &[ &str ]
    )
    -> Self
    {
        let mut storage = Storage::new( self.log.clone() );
        for ( id, (domain, actor, content) ) in &self.facts
        {
            if domains.contains( &domain.as_str() )
            {
                storage.facts.insert
                (
                    id.clone(),
                    (
                        domain.clone(),
                        actor.clone(),
                        content.clone()
                    )
                );
            }
        }
        storage
    }



    /*
        Generate new unique ID (microseconds timestamp)
    */
    pub fn generate_id( &mut self )
    -> String
    {
        use std::time::{ SystemTime, UNIX_EPOCH };

        let now = SystemTime::now()
        .duration_since( UNIX_EPOCH )
        .unwrap_or_default()
        .as_micros();

        let id = if now <= self.last_generated_id
        {
            self.last_generated_id + 1
        }
        else
        {
            now
        };

        self.last_generated_id = id;

        id.to_string()
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
        Return state
    */
    pub fn get_state_mut( &mut self )
    -> &mut State
    {
        &mut self.state
    }



    /*
        Clear all facts from storage
    */
    #[allow(dead_code)]
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
        /* Domain */
        String,
        /* Actor */
        String,
        /* Сontent */
        String
    )
    {
        if let Some(( domain, actor, content )) = self.facts.get( id )
        {
            (
                domain.clone(),
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
            ( String::new(), String::new(), String::new() )
        }
    }



    /*
        Insert new fact
        Returns generated ID
    */
    pub fn insert
    (
        &mut self,
        /* Id */
        id: &str,
        /* Domainn */
        domain: &str,
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
            if self.check_access( domain, 'i' ) || no_right
            {
                let id = if id == ""
                {
                    self.generate_id()
                }
                else
                {
                    id.to_string()
                };
                self.facts.insert
                (
                    id,
                    (
                        domain.to_string(),
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
                            "domain": domain,
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
        /* Domain */
        domain: &str,
        /* Actor */
        actor: &str,
        /* Content of fact */
        content: &str,
        /* True for disable check rights */
        ignore_access: bool
    )
    -> &mut Self
    {
        if self.get_state().is_ok()
        {
            if let Some(( old_domain, _, _ )) = self.get_by_id( id )
            {
                if
                    ignore_access
                    ||
                    (
                        domain == old_domain &&
                        self.check_access( domain, 'u' )
                    )
                    ||
                    (
                        domain != old_domain &&
                        self.check_access( domain, 'i' ) &&
                        self.check_access( &old_domain, 'd' )
                    )
                {
                    self.facts.insert
                    (
                        id.to_string(),
                        (
                            domain.to_string(),
                            actor.to_string(),
                            content.to_string()
                        )
                    );
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
            else
            {
                self.state.set_state
                (
                    "fact-not-found-for-update",
                    json!({ "id": id })
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
            if let Some(( old_domain, _, _ )) = self.get_by_id( id )
            {
                if no_right || self.check_access( &old_domain, 'd' )
                {
                    self.facts.shift_remove( id );
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
            else
            {
                self.state.set_state
                (
                    "fact-not-found-for-delete",
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
        if let Some(( domain, actor, content )) = self.facts.get( id )
        {
            return format!
            (
                "{}|{}|{}|{}\n\n{}\n\n",
                self.fact_delimiter,
                domain,
                actor,
                id,
                content
            );
        }
        String::new()
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


    #[allow(dead_code)]
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
    )
    -> bool
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



    pub fn set_access
    (
        &mut self,
        domain: &str,
        rights: &str
    )
    -> &mut Self
    {
        self.access.insert
        (
            domain.to_string(),
            rights.to_string()
        );
        self
    }


    #[allow(dead_code)]
    pub fn get_access
    (
        &self,
        domain: &str
    )
    -> String
    {
        self.access
        .get( domain )
        .cloned()
        .unwrap_or_else( || "".to_string() )
    }



    pub fn check_access
    (
        &self,
        domain: &str,
        right: char
    )
    -> bool
    {
        self.access
        .get( domain )
        .map(|rights| rights.contains(right))
        .unwrap_or(false)
    }



    /*
        Get fact by ID
        Returns Option with (domain, actor, content)
    */
    pub fn get_by_id
    (
        &self,
        id: &str
    )
    -> Option
    <&
    (
        /*domain actor content */
        String, String, String
    )
    >
    {
        self.facts.get(id)
    }
}

