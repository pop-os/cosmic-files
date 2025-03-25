#[cfg(feature = "jemalloc")]
use tikv_jemallocator::Jemalloc;

#[cfg(feature = "jemalloc")]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

use lexopt::{Arg, Parser};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = Parser::from_env();

    // Parse the arguments
    while let Some(arg) = parser.next()? {
        match arg {
            Arg::Short('h') | Arg::Long("help") => {
                print_help(env!("CARGO_PKG_VERSION"), env!("VERGEN_GIT_SHA"));
                return Ok(());
            }
            Arg::Short('v') | Arg::Long("version") => {
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
