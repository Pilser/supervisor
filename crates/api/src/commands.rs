//! Commands handle: custom command registration surface for plugins.

/// Plugin-facing handle for wrapper-driven commands.
#[derive(Debug, Default)]
pub struct CommandsApi {
    pub queue: commands::CommandQueue,
}

impl CommandsApi {
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse and enqueue an incoming command.
    pub fn receive(&mut self, raw: &str, priority: commands::Priority) -> core::Result<()> {
        let cmd = commands::listener::parse_command(raw)?;
        self.queue.enqueue(cmd, priority);
        Ok(())
    }
}
