//! PTY-based terminal implementation for interactive commands
//!
//! Provides proper pseudo-terminal support for commands like interactive Claude

use anyhow::{anyhow, Result};
use portable_pty::{CommandBuilder, MasterPty, NativePtySystem, PtySize, PtySystem};
use std::io::Write;
use std::sync::Arc;
use std::thread;
use tokio::sync::{mpsc, Mutex};

/// PTY process handle for interactive terminal sessions
pub struct PtyProcess {
    writer: Arc<Mutex<Box<dyn std::io::Write + Send>>>,
    child: Arc<Mutex<Box<dyn portable_pty::Child + Send + Sync>>>,
}

impl PtyProcess {
    /// Spawn a new PTY process
    pub fn spawn(
        command: &str,
        args: &[&str],
        working_dir: &str,
    ) -> Result<(Self, mpsc::Receiver<String>)> {
        let pty_system = NativePtySystem::default();

        // Create a new pty
        let pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| anyhow!("Failed to open PTY: {}", e))?;

        // Spawn the command
        let mut cmd = CommandBuilder::new(command);
        cmd.args(args);
        cmd.cwd(working_dir);

        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| anyhow!("Failed to spawn command: {}", e))?;

        // Set up output channel
        let (tx, rx) = mpsc::channel(100);

        // Clone the reader for the output thread
        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| anyhow!("Failed to clone PTY reader: {}", e))?;

        // Get writer before moving master
        let writer = pair
            .master
            .take_writer()
            .map_err(|e| anyhow!("Failed to take PTY writer: {}", e))?;

        // Spawn output reader thread (use std::thread for blocking I/O)
        thread::spawn(move || {
            let mut buf = vec![0u8; 4096];

            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        let output = String::from_utf8_lossy(&buf[..n]).to_string();
                        // Use blocking send since we're in a sync thread
                        let _ = tx.blocking_send(output);
                    }
                    Err(_) => break,
                }
            }
        });

        Ok((
            PtyProcess {
                writer: Arc::new(Mutex::new(writer)),
                child: Arc::new(Mutex::new(child)),
            },
            rx,
        ))
    }

    /// Write input to the PTY  
    pub async fn write(&self, input: &str) -> Result<()> {
        let mut writer = self.writer.lock().await;
        writer
            .write_all(input.as_bytes())
            .map_err(|e| anyhow!("Failed to write to PTY: {}", e))?;
        writer
            .flush()
            .map_err(|e| anyhow!("Failed to flush PTY: {}", e))
    }

    /// Check if the process is still running
    pub async fn is_running(&self) -> bool {
        let mut child = self.child.lock().await;
        match child.try_wait() {
            Ok(None) => true, // Still running
            _ => false,       // Exited or error
        }
    }

    /// Kill the process
    pub async fn kill(&self) -> Result<()> {
        let mut child = self.child.lock().await;
        child
            .kill()
            .map_err(|e| anyhow!("Failed to kill process: {}", e))
    }
}
