use gio::prelude::*;
use std::env;

fn main() {
    let uri = env::args().nth(1).expect("no uri provided");
    let file = gio::File::for_uri(&uri);
    for entry_res in file
        .enumerate_children("*", gio::FileQueryInfoFlags::NONE, gio::Cancellable::NONE)
        .unwrap()
    {
        let entry = entry_res.unwrap();
        println!("{:?}", entry.display_name());
        for attribute in entry.list_attributes(None) {
            println!(
                "  {:?}: {:?}",
                attribute,
                entry.attribute_as_string(&attribute)
            );
        }

        //TODO: what is the best way to resolve shortcuts?
        let child = if let Some(target_uri) =
            entry.attribute_string(gio::FILE_ATTRIBUTE_STANDARD_TARGET_URI)
        {
            gio::File::for_uri(&target_uri)
        } else {
            file.child(entry.name())
        };
        println!("{:?}", child.uri());
    }
}
