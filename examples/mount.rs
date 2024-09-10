use gio::prelude::*;
use std::env;

fn main() {
    let uri = env::args().nth(1).expect("no uri provided");
    let context = glib::MainContext::new();
    context.block_on(async {
        let mount_op = gio::MountOperation::new();
        mount_op.connect_ask_password(|mount_op, message, default_user, default_domain, flags| {
            println!(
                "{}, {}, {}, {:?}",
                message, default_user, default_domain, flags
            );
            mount_op.set_anonymous(true);
            mount_op.reply(gio::MountOperationResult::Handled);
        });
        let file = gio::File::for_uri(&uri);
        let res = file
            .mount_enclosing_volume_future(gio::MountMountFlags::empty(), Some(&mount_op))
            .await;
        println!("{:?}", res);
    });
}
