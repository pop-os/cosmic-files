#[cfg(feature = "jemalloc")]
use tikv_jemallocator::Jemalloc;

#[cfg(feature = "jemalloc")]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

use clap_lex::RawArgs;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let raw_args = RawArgs::from_args();
    let mut cursor = raw_args.cursor();

    // Parse the arguments
    while let Some(arg) = raw_args.next_os(&mut cursor) {
        match arg.to_str() {
            Some("--help") | Some("-h") => {
                print_help(env!("CARGO_PKG_VERSION"), env!("VERGEN_GIT_SHA"));
                return Ok(());
            }
            Some("--version") | Some("-v") => {
		println!(
                    "cosmic-files {} (git commit {})",
                    env!("CARGO_PKG_VERSION"),
                    env!("VERGEN_GIT_SHA")
                );
                return Ok(());
            }
            _ => {}
        }
    }
    cosmic_files::main()
}

fn print_help(version: &str, git_rev: &str) {
    println!(
        r#"cosmic-files {version} (git commit {git_rev})
System76 <info@system76.com>
	    
Designed for the COSMIC™ desktop environment, cosmic-files is a libcosmic-based file manager.
	    
Project home page: https://github.com/pop-os/cosmic-files
	    
Options:
  -h, --help     Show this message
  -v, --version  Show the version of cosmic-files"#
    );
}
