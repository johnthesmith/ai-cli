/*
    SPDX-License-Identifier: MIT
    SPDX-FileCopyrightText: 2026 Still Swamp
*/

impl Ai
{
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
        let files = &self.app.config[ type_operation ]
        .get_string_list( Vec::new() );
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
}
