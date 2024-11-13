use cosmic_files::operation::{recursive::Context, ReplaceResult};
use std::{error::Error, io};

fn main() -> Result<(), Box<dyn Error>> {
    let mut context = Context::new()
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

    context.recursive_copy("test/a", "test/b")?;
    context.recursive_move("test/b", "test/c")?;
    Ok(())
}
