pub fn parse_filename(args: &[String]) -> Result<String, &'static str> {
    if args.len() < 2 {
        return Err("not enought argument");
    }
    let filename = args[1].clone();
    return Ok(filename);
}
