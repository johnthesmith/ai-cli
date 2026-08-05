/*
    SPDX-License-Identifier: MIT
    SPDX-FileCopyrightText: 2026 Still Swamp
*/


impl Ai
{
    /*
        Return list of files
    */
    pub fn get_files
    (
        &self,
        val: &str
    )
    -> Vec<String>
    {
        let mut result = Vec::new();

        /* Expend path */
        let val = &core::expand_path( val );

        let path = std::path::Path::new( val );

        let dir = if val.ends_with( '/' )
        {
            path.to_path_buf()
        }
        else if let Some( parent ) = path.parent()
        {
            parent.to_path_buf()
        }
        else
        {
            std::path::PathBuf::from( "." )
        };

        if let Ok( entries ) = std::fs::read_dir( &dir )
        {
            for entry in entries.flatten()
            {
                if let Ok(file_type) = entry.file_type()
                {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let full_path = dir.join( &name );
                    let display = full_path.to_string_lossy().to_string();
                    if file_type.is_dir()
                    {
                        result.push( format!( "{}/", display ));
                    }
                    else
                    {
                        result.push( display );
                    }
                }
            }
        }
        result
    }
}
