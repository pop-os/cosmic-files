use std::{fs, io, path::Path};

use super::Controller;

// Special reader just for operations, handling cancel and progress
pub struct OpReader {
    file: fs::File,
    metadata: fs::Metadata,
    current: u64,
    controller: Controller,
}

impl OpReader {
    pub fn new<P: AsRef<Path>>(path: P, controller: Controller) -> io::Result<Self> {
        let file = fs::File::open(&path)?;
        let metadata = file.metadata()?;
        Ok(Self {
            file,
            metadata,
            current: 0,
            controller,
        })
    }
}

impl io::Read for OpReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.controller
            .check()
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

        let count = self.file.read(buf)?;
        self.current += count as u64;

        let progress = self.current as f32 / self.metadata.len() as f32;
        self.controller.set_progress(progress);

        Ok(count)
    }
}
