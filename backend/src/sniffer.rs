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
                "-Y", "websocket and tcp.port == 443", // Filter: Only income websockets
                "-T", "fields",                           // Format: Specific fields
                "-e", "websocket.payload.text",           // Fields: Clear JSON (for incoming)
                "-e", "websocket.payload",      // Raw bytes (for outcoming)
                "-e", "websocket.masking_key",  // Masking key (for outcoming)
                "-E", "separator=,",            // Separator for convenient parsing
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
        let bytes_read = self
            .reader
            .read_line(&mut buffer)
            .map_err(SnifferError::NonValidSequence)?;

        if bytes_read == 0 {
            return Err(SnifferError::ProcessTerminated);
        }

        Ok(buffer)
    }

    pub fn wait_child(&mut self) {
        let _ = self.child.wait(); // Waiting for process end, if it will fail
    }

    /// Decrypting outcome WebSocket Payload by RFC 6455
    pub fn unmask_websocket_payload(
        payload_hex: &str, mask_hex: &str,
    ) -> Result<String, SnifferError> {
        // TShark often outputs hex with colons (e.g., "1a:2b:3c"), so we need to remove them before decoding
        let clean_payload = payload_hex.replace(":", "");
        let clean_mask = mask_hex.replace(":", "");

        // Decoding HEX-Strings into bytes vec
        let payload_bytes =
            hex::decode(&clean_payload).map_err(SnifferError::HexDecode)?;
        let mask_bytes = hex::decode(&clean_mask).map_err(SnifferError::HexDecode)?;

        if mask_bytes.len() != 4 {
            return Err(SnifferError::MaskingKeyLength(mask_bytes.len()));
        }

        // Using XOR-mask
        #[allow(clippy::indexing_slicing)]
        let unmasked_bytes: Vec<u8> = payload_bytes
            .into_iter()
            .enumerate()
            .map(|(i, byte)| byte ^ mask_bytes[i % 4])
            .collect();

        // Trying to get json from possibly decoded bytes
        String::from_utf8(unmasked_bytes).map_err(|_| SnifferError::DecodingWithMask)
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

    #[error("Process terminated. 0 bytes read from reader.")]
    ProcessTerminated,

    #[error("Failed to decode input from HEX to bytes. {0}")]
    HexDecode(hex::FromHexError),

    #[error("Masking key must be exactly 4 bytes. Provided mask length: {0}")]
    MaskingKeyLength(usize),

    #[error("Failed to convert 'decoded' payload with mask into Json String.")]
    DecodingWithMask,
}
