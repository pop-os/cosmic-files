// This launches the desktop mode as a regular window for easier testing.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    cosmic_files::desktop()
}
