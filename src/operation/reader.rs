use cosmic::iced::futures::{channel::mpsc::Sender, executor, SinkExt};
use std::{
    fs, io,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tokio::sync::Mutex;

use crate::{app::Message, fl};

// Special reader just for operations, handling cancel and progress
pub struct OpReader {
    file: fs::File,
    metadata: fs::Metadata,
    current: u64,
    id: u64,
    msg_tx: Arc<Mutex<Sender<Message>>>,
    cancelled: Arc<AtomicBool>,
}

impl OpReader {
    pub fn new<P: AsRef<Path>>(
        path: P,
        id: u64,
        msg_tx: Arc<Mutex<Sender<Message>>>,
        cancelled: Arc<AtomicBool>,
    ) -> io::Result<Self> {
        let file = fs::File::open(&path)?;
        let metadata = file.metadata()?;
        Ok(Self {
            file,
            metadata,
            current: 0,
            id,
            msg_tx,
            cancelled,
        })
    }
}

impl io::Read for OpReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.cancelled.load(Ordering::SeqCst) {
            return Err(io::Error::new(io::ErrorKind::Other, fl!("cancelled")));
        }

        let count = self.file.read(buf)?;
        self.current += count as u64;

        let progress = self.current as f32 / self.metadata.len() as f32;
        executor::block_on(async {
            let _ = self
                .msg_tx
                .lock()
                .await
                .send(Message::PendingProgress(self.id, 100.0 * progress))
                .await;
        });

        Ok(count)
    }
}
