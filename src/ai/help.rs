include!( "help_template.rs" );

impl Ai
{
    /*
        Help utility
    */
    fn help( &mut self )
    -> &mut Self
    {
        println!
        (
            "{}",
            CONTENT.replace( "%version%", &self.get_version() )
        );
        self
    }
}
