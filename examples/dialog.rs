fn main() -> Result<(), Box<dyn std::error::Error>> {
    match cosmic_files::dialog()? {
        Some(paths) => {
            for path in paths {
                println!("{}", path.display());
            }
        }
        None => {}
    }
    Ok(())
}
