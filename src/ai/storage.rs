/*
    SPDX-License-Identifier: MIT
    SPDX-FileCopyrightText: 2026 Still Swamp
*/

/*
    Storage interface for CRUD operations on facts
    Each fact = one block in text file

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
    /* In-memory blocks loaded once  id -> actor,content */
    pub blocks: BTreeMap
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
    pub fn new()
    -> Self
    {
        Self
        {
            allow_create: true,
            allow_delete: true,
            allow_update: true,
            state: State::ok(),
            blocks: BTreeMap::new(),
            fact_delimiter: "-=*=-".to_string()
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
        let lines: Vec<&str> = content.lines().collect();
        if !lines.is_empty() 
        {
            self.fact_delimiter = lines[0].trim().to_string();
            if !self.fact_delimiter.is_empty()
            {

                let mut blocks = Vec::new();
                let mut current_block = Vec::new();
                let mut i = 1; // skip first line (delimiter)
                while i < lines.len() 
                {
                    let line = lines[i];                 
                    /* Check if line is exactly the delimiter (no extra chars) */
                    if line == self.fact_delimiter 
                    {
                        /* End of current block */
                        if !current_block.is_empty() 
                        {
                            blocks.push( current_block.join( "\n" ));
                            current_block.clear();
                        }
                    } 
                    else 
                    {
                        current_block.push(line);
                    }
                    
                    i += 1;
                }

                /* Last block */
                if !current_block.is_empty() 
                {
                    blocks.push(current_block.join( "\n" ));
                }
            
                for block in blocks
                {
                    let block = block.trim();
                    if !block.is_empty()
                    {
                        let lines: Vec<&str> = block.lines().collect();
                        if lines.len() >= 4
                        {
                            let id = lines[0].trim().to_string();
                            let origin = lines[1].trim().to_string();
                            let action = lines[2].trim().to_string();
                            let actor = lines[3].trim().to_string();
                            let content = if lines.len() >=5 
                            {
                                lines[ 4.. ].join( "\n" ).trim().to_string() 
                            }
                            else
                            {
                                "".to_string()
                            };

                            let final_id = if id == "-" { self.generate_id() } else { id };
                            self.blocks.insert( final_id, ( origin, action, actor, content ));
                        }
                    }
                }
            }
        }
    }



    /*
        Parse blocks from string content
        Returns number of blocks parsed
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
            self.fact_delimiter = lines[0].trim().to_string();
            if !self.fact_delimiter.is_empty()
            {

                let mut blocks = Vec::new();
                let mut current_block = Vec::new();
                let mut i = 1; // skip first line (delimiter)
                while i < lines.len() 
                {
                    let line = lines[i];                 
                    /* Check if line is exactly the delimiter (no extra chars) */
                    if line == self.fact_delimiter 
                    {
                        /* End of current block */
                        if !current_block.is_empty() 
                        {
                            blocks.push( current_block.join( "\n" ));
                            current_block.clear();
                        }
                    } 
                    else 
                    {
                        current_block.push(line);
                    }
                    
                    i += 1;
                }

                /* Last block */
                if !current_block.is_empty() 
                {
                    blocks.push(current_block.join( "\n" ));
                }
            
                for block in blocks
                {
                    let block = block.trim();
                    if !block.is_empty()
                    {
                        let lines: Vec<&str> = block.lines().collect();
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
                                        content = lines[ 1.. ].join( "\n" ).trim().to_string();
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
                                        content = lines[ 1.. ].join( "\n" ).trim().to_string();
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
                                        actor = crate::ai::ASSISTANT.to_string();
                                    };                                                                     
                                },


                                "change" | "update"  =>
                                {
                                    if lines.len() > 3
                                    {
                                        action = "change".to_string();
                                        id = lines[ 1 ].to_string();
                                        actor = lines[ 2 ].to_string();
                                        content = lines[ 3.. ]
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
                            self.blocks.insert
                            (
                                id, 
                                ( origin, action, actor, content )
                            );
                        }
                    }
                }
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
        let content = match fs::read_to_string(path)
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

        self.blocks.clear();
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
        /* Origin of fact */
        origin: &str,
        /* Action */
        action: &str,
        /* Actor */
        actor: &str,
        /* Content of fact */
        content: &str
    )
    -> &mut Self
    {
        if self.allow_create
        {
            let id = self.generate_id();
            self.blocks.insert
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
                "storage-create-not-allowed",
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
        /* Origin of fact */
        origin: &str,
        /* Action */
        action: &str,
        /* Actor */
        actor: &str,
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
                "storage-update-not-allowed",
                json!
                (
                    {
                        "id": id,
                        "origin": origin,
                        "action": action,
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
            self.blocks.remove( id );
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
        if let Some(( origin, action, actor, content )) = self.blocks.get( id )
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
        Get fact by ID as string with delimiter
    */
    pub fn to_string_by_id
    ( 
        &self, 
        id: &str
    )
    -> String
    {
        if let Some(( origin, action, actor, content )) = self.blocks.get( id )
        {
            return format!
            (
                "{}\n{}\n{}\n{}\n{}\n\n{}\n\n",
                self.fact_delimiter,
                id,
                origin,
                action,
                actor,
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
        if let Some(( origin, _, actor, content )) = self.blocks.get( id )
        {
            return format!
            (
                "{}\n{}\n{}\n{}\n\n{}\n\n",
                self.fact_delimiter,
                origin,
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
        for( id, _ ) in &self.blocks
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
        for( id, _ ) in &self.blocks
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
        self.blocks.contains_key(id)
    }
}
