impl Ai
{
    /*
        Check right
    */
    fn check_access
    (
        rights: &str,
        right: &str
    )
    -> bool
    {
        rights.contains( right )
    }
}
