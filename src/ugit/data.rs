const GIT_DIR: &str = ".ugit";

pub fn init() -> std::io::Result<()> {
    std::fs::create_dir(GIT_DIR)?;
    Ok(())
}
