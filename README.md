# TH3AL3X · COSMIC Files Custom

Personal modifications for Pop!_OS 24.04 / COSMIC, based on upstream commit `3cc4b80` (COSMIC Files 1.11.0).

- **Transparent desktop labels:** removes the background behind unselected desktop icon names. Selection and hover highlights remain.
- **Bottom-right Show Desktop button:** a small 10 × 36 logical-pixel strip on each monitor. Left-click minimizes visible windows on active workspaces; click again restores the captured windows. Already minimized windows are left alone. Uses COSMIC Wayland protocols, with no pointer polling or animations.

Installation, rollback, limitations and credits: **[local-mods/README.md](local-mods/README.md)**.

This is an independent customization, not an official System76 release. Distribution updates refresh the preserved original applet; rebuild the custom applet to receive upstream fixes.

---

# cosmic-files
File manager for the COSMIC desktop environment

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
