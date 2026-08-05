/*
    SPDX-License-Identifier: MIT
    SPDX-FileCopyrightText: 2026 Still Swamp
*/

/*
    Commands
*/

impl Ai
{
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

}
