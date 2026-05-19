use std::io::{BufRead, BufReader};
use std::process::{Child, ChildStdout, Command, Stdio};
use thiserror::Error;

pub struct Sniffer {
    child: Child,                   // TShark Child Process
    reader: BufReader<ChildStdout>, // Standard output reader for TShark
}

impl Sniffer {
    pub fn start(interface: &str, keylog_path: &str) -> Result<Self, SnifferError> {
        let mut child = Command::new("tshark")
            .args([
                "-l", // Line-buffered
                "-i", interface, // Interface
                "-o", &format!("tls.keylog_file:{}", keylog_path), // Path to the file with TLS keys
                "-Y", "websocket and tcp.srcport == 443", // Filter: Only income websockets
                "-T", "fields",                           // Format: Specific fields
                "-e", "websocket.payload.text",           // Fields: Clear JSON
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::null()) // Ignoring technical spam
            .spawn()
            .map_err(SnifferError::TSharkRun)?;

        log::info!(
            "TShark started. Listening on {}. PID: {}",
            interface,
            child.id()
        );

        let stdout = child.stdout.take().ok_or(SnifferError::Stdout)?;
        let reader = BufReader::new(stdout);

        Ok(Self { child, reader })
    }

    pub fn read(&mut self) -> Result<String, SnifferError> {
        let mut buffer = String::new();
        let _bytes_read = self
            .reader
            .read_line(&mut buffer)
            .map_err(SnifferError::NonValidSequence)?;

        Ok(buffer)
    }

    pub fn wait_child(&mut self) {
        let _ = self.child.wait(); // Waiting for process end, if it will fail
    }
}

#[derive(Debug, Error)]
pub enum SnifferError {
    #[error(
        "Failed to run TShark. Please check it is installed and have rights (`wireshark` group). {0}"
    )]
    TSharkRun(std::io::Error),

    #[error("Failed to take standard output from child process.")]
    Stdout,

    #[error("Failed to read line from TShark output. {0}")]
    NonValidSequence(std::io::Error),
}
