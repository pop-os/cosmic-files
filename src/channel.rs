// Copyright 2025 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use std::{
    collections::VecDeque,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

/// Create a channel backed by `tokio::sync::Notify` and a sync mutex with a vec deque.
pub fn channel<Message>() -> (Sender<Message>, Receiver<Message>) {
    let channel = Arc::new(Channel {
        queue: Mutex::new(VecDeque::default()),
        notify: tokio::sync::Notify::const_new(),
        closed: AtomicBool::new(false),
    });

    (Sender(channel.clone()), Receiver(channel))
}

/// A channel backed by `tokio::sync::Notify` and a sync mutex with a vec deque.
struct Channel<Message> {
    pub(self) queue: Mutex<VecDeque<Message>>,
    /// Set when a new message has been stored.
    pub(self) notify: tokio::sync::Notify,
    /// Set when the receiver is dropped.
    pub(self) closed: AtomicBool,
}

pub struct Sender<Message>(Arc<Channel<Message>>);

impl<Message> Sender<Message> {
    pub fn send(&self, message: Message) {
        self.0.queue.lock().unwrap().push_back(message);
        self.0.notify.notify_one();
    }
}

impl<Message> Drop for Sender<Message> {
    fn drop(&mut self) {
        self.0.closed.store(true, Ordering::SeqCst);
        self.0.notify.notify_one();
    }
}

pub struct Receiver<Message>(Arc<Channel<Message>>);

impl<Message> Receiver<Message> {
    /// Returns a value until the sender is dropped.
    pub async fn recv(&self) -> Option<Message> {
        loop {
            {
                let mut queue = self.0.queue.lock().unwrap();
                if let Some(value) = queue.pop_front() {
                    if queue.capacity() - queue.len() > 32 {
                        let capacity = queue.len().next_power_of_two();
                        queue.shrink_to(capacity);
                    }
                    drop(queue);
                    return Some(value);
                }
            }

            if self.0.closed.load(Ordering::SeqCst) {
                return None;
            }

            self.0.notify.notified().await;
        }
    }

    pub fn try_recv(&self) -> Option<Message> {
        self.0.queue.lock().unwrap().pop_front()
    }
}
