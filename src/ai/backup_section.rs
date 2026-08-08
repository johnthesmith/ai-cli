/*
    Backup section
*/

use std::fs;
use sha2::{ Sha256, Digest };
use std::path::Path;



/*
    Compute SHA256 hash of input string
*/
fn string_to_sha256
(
    input: &str
)
-> String
{
    hex::encode(Sha256::digest( input.as_bytes()) )
}



impl Ai
{
    /*
        Return backup path
    */
    fn get_backup_path
    (
        &self,
        origin_file: &str
    )
    -> String
    {
        /* Resolve absolute path */
        let abs_path = core::get_abs_path( origin_file );

        /* Compute SHA256 hash of absolute path */
        let hash_str = string_to_sha256( &abs_path );

        core::expand_path
        (
            &self.get_config_str
            (
                &[ "backup-path" ],
                "%profile-path%/backup/%origin-file-hash%"
            )
            .replace( "%profile-path%", &self.get_profile_path())
            .replace( "%origin-file-hash%", &hash_str )
            .replace( "default", &self.get_chat())
        )
    }



    /*
        Backup restore
    */
    fn backup_restore
    (
        &mut self,
        /* File name */
        origin_file: &str
    )
    {
        /* Build backup path */
        let backup_file = self.get_backup_path( origin_file );

        /* Check if backup exists */
        if !Path::new( &backup_file ).exists()
        {
            self.app.state.set_state
            (
                "backup-restore-failed",
                json!
                (
                    {
                        "message": "Backup not found",
                        "backup-file": backup_file
                    }
                )
            );
        }
        else
        {
            /* Read backup content */
            let content = match fs::read( &backup_file )
            {
                Ok( c ) => c,
                Err( e ) =>
                {
                    self.app.state.set_state
                    (
                        "backup-restore-failed",
                        json!
                        (
                            {
                                "message": "Failed to read backup",
                                "error": e.to_string(),
                                "backup-file": backup_file
                            }
                        )
                    );

                    return;
                }
            };

            /* Write content back to origin file */
            if fs::write( &origin_file, content ).is_err()
            {
                self.app.state.set_state
                (
                    "backup-restore-failed",
                    json!
                    (
                        {
                            "message": "Failed to restore file",
                            "origin-file": origin_file,
                            "backup-file": backup_file
                        }
                    )
                );
            }
        }
    }



    /*
        Backup file
    */
    pub fn backup_file
    (
        &mut self,
        /* File name */
        origin_file: &str
    )
    -> &Self
    {

        /* Build backup file */
        let backup_file = self.get_backup_path( origin_file );

        /* Ensure backup directory exists */
        if let Err( e ) = core::ensure_directory( &backup_file )
        {
            self.app.state.set_state
            (
                "backup-failed",
                json!
                (
                    {
                        "message": "Failed to create directory for backup",
                        "error": &e,
                        "path": backup_file
                    }
                )
            );
        }
        else
        {
            /* Read source file */
            let content = match fs::read( &origin_file )
            {
                Ok( c ) => c,
                Err( e ) =>
                {
                    self.app.state.set_state
                    (
                        "backup-failed",
                        json!
                        (
                            {
                                "message": "Failed to read content",
                                "error": e.to_string(),
                                "origin-file": origin_file
                            }
                        )
                    );

                    return self;
                }
            };

            /* Write backup content */
            if fs::write( &backup_file, content ).is_err()
            {
                self.app.state.set_state
                (
                    "backup-failed",
                    json!
                    (
                        {
                            "message": "Backup failed",
                            "origin-file": &origin_file,
                            "backup-file": backup_file
                        }
                    )
                );
            }
        }
        self
    }
}
