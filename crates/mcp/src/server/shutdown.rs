//! Cooperative shutdown signalling for the McpServer.

use tokio::sync::watch;

/// The signal sender side, held by the server owner.
#[derive(Debug, Clone)]
pub struct Shutdown {
    tx: watch::Sender<bool>,
}

/// A receiver that can be awaited to detect a shutdown request.
#[derive(Debug)]
pub struct ShutdownSignal {
    rx: watch::Receiver<bool>,
}

impl Shutdown {
    /// Create a new shutdown channel pair.
    pub fn new() -> (Shutdown, ShutdownSignal) {
        let (tx, rx) = watch::channel(false);
        (Shutdown { tx }, ShutdownSignal { rx })
    }

    /// Ask the server to stop gracefully.
    pub fn signal(&self) {
        let _ = self.tx.send(true);
    }

    /// Whether a signal has already been sent.
    pub fn requested(&self) -> bool {
        *self.tx.borrow()
    }
}

impl ShutdownSignal {
    /// Wait until a shutdown signal is received, blocking forever otherwise.
    pub async fn wait(&mut self) {
        if *self.rx.borrow() {
            return;
        }
        let mut rx = self.rx.clone();
        let _ = rx.changed().await;
    }

    /// Wait until a signal is received or until canceled externally.
    pub async fn wait_cancellable(&mut self) -> bool {
        if *self.rx.borrow() {
            return true;
        }
        let mut rx = self.rx.clone();
        loop {
            if rx.changed().await.is_ok() && *rx.borrow() {
                return true;
            }
        }
    }

    /// True if the signal has already been raised.
    pub fn requested(&self) -> bool {
        *self.rx.borrow()
    }
}

impl Default for Shutdown {
    fn default() -> Self {
        Self::new().0
    }
}