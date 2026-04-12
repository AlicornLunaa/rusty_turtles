pub fn read_turtle_script() -> (u64, String) {
    // This returns the script as a string and the version number
    let content = std::fs::read_to_string("./turtle/turtle.lua").unwrap();
    let first_line = content.lines().next().unwrap();
    let (_, version) = first_line.split_at(first_line.find("=").unwrap() + 1);
    let version: u64 = version.trim().parse().unwrap();
    (version, content)
}