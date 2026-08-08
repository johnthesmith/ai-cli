/*
    SPDX-License-Identifier: MIT
    SPDX-FileCopyrightText: 2026 Still Swamp
*/

/*
    Main AI module
*/
mod providers;
mod storage;

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
    fact_db: Storage,

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

    /*
        File write resolving table by fact
        id fact: file path
    */
    write_translation: BTreeMap < String, String >,

    /* Colorize */
    colorize: bool
}


/*
    Including part of code for ai.rs
*/

include!( "ai/create.rs" );
include!( "ai/get_app.rs" );

include!( "ai/init.rs" );
include!( "ai/run.rs" );

include!( "ai/get_version.rs" );
include!( "ai/out_info.rs" );
include!( "ai/help.rs" );

include!( "ai/get_files.rs" );
include!( "ai/insert_files.rs" );
include!( "ai/compile_prompt.rs" );
include!( "ai/handle_chat_response.rs" );
include!( "ai/completion.rs" );

include!( "ai/prompt_section.rs" );
include!( "ai/memory_section.rs" );
include!( "ai/model_section.rs" );
include!( "ai/provider_section.rs" );
include!( "ai/profile_section.rs" );
include!( "ai/chat_section.rs" );
include!( "ai/commands.rs" );
include!( "ai/history_section.rs" );
include!( "ai/backup_section.rs" );

include!( "ai/mnemo.rs" );
include!( "ai/token.rs" );
include!( "ai/config.rs" );
include!( "ai/check_access.rs" );

