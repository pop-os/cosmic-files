use cosmic_files::operation::{recursive::Context, Controller, ReplaceResult};
use std::{error::Error, io, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let mut context = Context::new(Controller::default())
        .on_progress(|op, progress| {
            println!("{:?}: {:?}", op.to, progress);
        })
        .on_replace(|op| {
            println!("replace {:?}? (y/N)", op.to);
            let mut line = String::new();
            match io::stdin().read_line(&mut line) {
                Ok(_) => {
                    if line == "y" {
                        ReplaceResult::Replace(false)
                    } else {
                        ReplaceResult::Skip(false)
                    }
                }
                Err(err) => {
                    eprintln!("failed to read stdin: {}", err);
                    ReplaceResult::Cancel
                }
            }
        });

    context.recursive_copy_or_move(
        vec![(PathBuf::from("test/a"), PathBuf::from("test/b"))],
        false,
    )?;
    context.recursive_copy_or_move(
        vec![(PathBuf::from("test/b"), PathBuf::from("test/c"))],
        true,
    )?;
    Ok(())
}
