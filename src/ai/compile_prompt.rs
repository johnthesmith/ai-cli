/*
    SPDX-License-Identifier: MIT
    SPDX-FileCopyrightText: 2026 Still Swamp
*/

impl Ai
{
    /*
        Build full prompt content from template and context
    */
    fn compile_prompt
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

        self.storage.parse( &prompt_content );

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
}
