use tokio::{sync::watch, task::JoinHandle};

pub struct Preview {
    pub frames: watch::Sender<Option<Vec<u8>>>,
    /// Held so the channel never closes while the page exists. Without it a capture with no
    /// browser watching would fail its first send and stop.
    pub keepalive: watch::Receiver<Option<Vec<u8>>>,
    pub task: Option<JoinHandle<()>>,
}

impl Default for Preview {
    fn default() -> Self {
        let (frames, keepalive) = watch::channel(None);

        Self {
            frames,
            keepalive,
            task: None,
        }
    }
}

impl Preview {
    pub fn has_viewers(&self) -> bool {
        self.frames.receiver_count() > 1
    }
}
