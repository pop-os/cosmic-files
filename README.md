# cosmic-files
File manager for the COSMIC desktop environment

## Platform Requirements

**⚠️ Linux Ecosystem Required**

This project must be compiled and run in a Linux environment. It depends on:
- Wayland display server protocol
- Linux-specific system libraries (libxkbcommon, etc.)
- freedesktop.org standards

**macOS and Windows are not supported** for building or running this application.

## Build the project from source

```sh
# Clone the project using `git`
git clone https://github.com/pop-os/cosmic-files
# Change to the directory that was created by `git`
cd cosmic-files
# Build an optimized version using `cargo`, this may take a while
cargo build --release
# Run the optimized version using `cargo`
cargo run --release
```

## Community and Contributing

The COSMIC desktop environment is maintained by System76 for use in Pop!_OS. A list of all COSMIC projects can be found in the
[cosmic-epoch](https://github.com/pop-os/cosmic-epoch) project's README. If you would like to discuss COSMIC and Pop!_OS, please
consider joining the [Pop!_OS Chat](https://chat.pop-os.org/). More information and links can be found on the
[Pop!_OS Website](https://pop.system76.com).

## License

This project is licensed under [GPLv3](LICENSE)
