use cosmic_files::operation::recursive::Method;
use cosmic_files::operation::{Controller, ReplaceResult, recursive::Context};
use std::{error::Error, io, path::PathBuf};

#[compio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut context = Context::new(Controller::default())
        .on_progress(|op, progress| {
            println!("{:?}: {:?}", op.to, progress);
        })
        .on_replace(|op| {
            Box::pin(async move {
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
            })
        });

    context
        .recursive_copy_or_move(
            vec![(PathBuf::from("test/a"), PathBuf::from("test/b"))],
            Method::Copy,
        )
        .await?;
    context
        .recursive_copy_or_move(
            vec![(PathBuf::from("test/b"), PathBuf::from("test/c"))],
            Method::Move {
                cross_device_copy: false,
            },
        )
        .await?;

    Ok(())
}
