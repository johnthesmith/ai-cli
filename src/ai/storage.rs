/*
    SPDX-License-Identifier: MIT
    SPDX-FileCopyrightText: 2026 Still Swamp
*/

/*
    Storage interface for CRUD operations on facts
    Each fact = one block in text file

    %block-delimiter%\n<id>\n<type>\n<actor>\n<action>\n<content>\n

    Blocks are loaded into memory and saved on demand
*/

use std::fs;
use serde_json::json;
use core::State;
use std::collections::BTreeMap;



pub struct Storage
{
    /* Allow create operations */
    allow_create: bool,
    /* Allow delete operations */
    allow_delete: bool,
    /* Allow update operations */
    allow_update: bool,
    /* Storage state */
    state: State,
    /* In-memory blocks loaded once  id -> actor,body */
    pub blocks: BTreeMap
    <
        /* id */
        String, 
        (
            /* type = history | memory | prompt */
            String, 
            /* actor = system | assistant | user */
            String,
            /* action = read | message | command | pool |clipboard | add | remove | delete */
            String,
            /* fact body */
            String
        )
    >, 
    delimiter: String
}



impl Storage
{
    /*
        Create new storage
    */
    pub fn new
    (
        /* Block delimiter (e.g., "\n\n") */
        delimiter: &str
    )
    -> Self
    {
        Self
        {
            delimiter: delimiter.to_string(),
            allow_create: true,
            allow_delete: true,
            allow_update: true,
            state: State::ok(),
            blocks: BTreeMap::new(),
        }
    }



    /*
        Parse blocks from string content
        Returns number of blocks parsed
    */
    pub fn parse
    (
        &mut self, 
        content: &str
    ) 
    {
        for block in content.split( &self.delimiter )
        {
            let block = block.trim();
            if !block.is_empty()
            {
                let lines: Vec<&str> = block.lines().collect();
                if lines.len() >= 4
                {
                    let id = lines[0].trim().to_string();
                    let typ = lines[1].trim().to_string();
                    let actor = lines[2].trim().to_string();
                    let action = lines[3].trim().to_string();
                    let body = if lines.len() >=5 
                    {
                        lines[4..].join("\n").trim().to_string() 
                    }
                    else
                    {
                        "".to_string()
                    };

                    let final_id = if id == "-" { self.generate_id() } else { id };
                    self.blocks.insert( final_id, (typ, actor, action, body));
                }
            }
        }
    }



    /*
        Load storage from file
        File format:
            delimiter line
            id line
            typ line
            actor line
            action line
            body line(s)
            delimiter line
            ...
    */
    pub fn load
    (
        &mut self, 
        path: &str
    )
    -> &mut Self
    {
        let content = match fs::read_to_string(path)
        {
            Ok(c) => c,
            Err(e) =>
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

        self.blocks.clear();
        self.parse(&content);
        
        self
    }



    /*
        Save storage to file
        File format:
            delimiter
            id
            actor
            body
            delimiter
            id
            actor
            body
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
        if let Err(e) = core::ensure_directory(path)
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
        Clear all blocks from storage
    */
    pub fn clear( &mut self )
    -> &mut Self
    {
        self.blocks.clear();
        self
    }


    /**************************************************************************
        CRUD
    */

    
    /*
        Create new fact
        Returns generated ID
    */
    pub fn create
    (
        &mut self,
        /* Type of fact history|memory|prompt*/
        typ: &str,
        /* Actor system|assistant|user*/
        actor: &str,
        /* Action = read | message | command | pool |clipboard | add | remove | delete */
        action: &str,
        /* Content of fact */
        content: &str
    ) -> &mut Self
    {
        if self.allow_create
        {
            let id = self.generate_id();
            self.blocks.insert
            (
                id.clone(), 
                (
                    typ.to_string(),                   
                    actor.to_string(), 
                    action.to_string(), 
                    content.to_string()                    
                )
            );
        }
        else
        {
            self.state.set_state
            (
                "storage-create-not-allowed",
                json!
                (
                    {
                        "actor": actor,
                        "content": content
                    }
                )
            );
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
        /* Type of fact history|memory|prompt*/
        typ: &str,
        /* Actor system|assistant|user*/
        actor: &str,
        /* Action = read | message | command | pool |clipboard | add | remove | delete */
        action: &str,
        /* Content of fact */
        content: &str
    )
    -> &mut Self
    {
        if self.allow_update
        {
            self.blocks.insert
            (
                id.to_string(), 
                (
                    typ.to_string(),                   
                    actor.to_string(), 
                    action.to_string(), 
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
                        "actor": actor,
                        "content": content
                    }
                )
            );
        }
        self
    }    



    /*
        Delete fact by ID
    */
    pub fn delete
    (
        &mut self,
        id: &str
    )
    -> &mut Self
    {
        if self.allow_delete
        {
            self.blocks.remove(id);
        }
        else
        {
            self.state.set_state
            (
                "storage-delete-not-allowed",
                json!({ "id": id })
            );
        }

        self
    }



    /*
        Select fact by ID
        Returns (actor, body) or empty string if not found
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
        /* Type */
        String,
        /* Actor (user, agent, etc.) */
        String,
        /* Action */
        String,
        /* Body (fact content) */
        String
    )
    {
        if let Some(( typ, actor, action, body )) = self.blocks.get( id )
        {
            ( 
                typ.clone(),
                actor.clone(), 
                action.clone(),
                body.clone()
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
        Get fact by ID as string with delimiter
    */
    pub fn to_string_by_id
    ( 
        &self, 
        id: &str
    )
    -> String
    {
        if let Some((typ, actor, action, body)) = self.blocks.get( id )
        {
            return format!
            (
                "{}\n{}\n{}\n{}\n{}\n{}\n\n",
                self.delimiter,
                id,
                typ,
                actor,
                action,
                body
            );
        }
        
        String::new()
    }



    /*
        Get all facts as single string with delimiter
    */
    pub fn to_string( &self )
    -> String
    {
        let mut content = String::new();

        for( id, _ ) in &self.blocks
        {
            content.push_str( &self.to_string_by_id( id ));
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
        self.allow_create = access.contains( 'c' );
        self.allow_update = access.contains( 'u' );
        self.allow_delete = access.contains( 'd' );
        self
    }


    
    pub fn get_access( &self )
    -> String
    {
        let mut access = String::new();
        if self.allow_create { access.push( 'c' ); }
        if self.allow_update { access.push( 'u' ); }
        if self.allow_delete { access.push( 'd' ); }
        access
    }
}
