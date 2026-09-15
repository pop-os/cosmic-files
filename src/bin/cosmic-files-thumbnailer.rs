// SPDX-License-Identifier: GPL-3.0-only

#[path = "../exe_thumbnailer.rs"]
mod exe_thumbnailer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    exe_thumbnailer::thumbnail_from_args(std::env::args_os().skip(1))
}
