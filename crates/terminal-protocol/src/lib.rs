//! Protocol and replay type boundary for Hera M1.

#![forbid(unsafe_code)]

/// Maximum payload bytes retained for M1 string controls.
///
/// The parser can accept longer OSC/DCS-like strings, but Hera keeps a tighter
/// protocol-level cap so unsupported payloads cannot grow without bound.
pub const M1_PAYLOAD_LIMIT_BYTES: usize = 4096;

/// A normalized parser action owned by Hera.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminalAction {
    Print(Printable),
    Control(C0Control),
    Escape(EscapeSequence),
    Csi(CsiSequence),
    Osc(OscCommand),
    Dcs(DcsCommand),
    Apc(StringControl),
    Pm(StringControl),
    Unsupported(UnsupportedSequence),
}

/// Printable text that reached the terminal state layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Printable {
    ch: char,
}

impl Printable {
    #[must_use]
    pub const fn new(ch: char) -> Self {
        Self { ch }
    }

    #[must_use]
    pub const fn ch(self) -> char {
        self.ch
    }
}

/// C0/C1 control byte normalized before terminal semantics are applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct C0Control {
    byte: u8,
    kind: C0ControlKind,
}

impl C0Control {
    #[must_use]
    pub const fn from_byte(byte: u8) -> Self {
        let kind = match byte {
            0x00 => C0ControlKind::Null,
            0x07 => C0ControlKind::Bell,
            0x08 => C0ControlKind::Backspace,
            0x09 => C0ControlKind::HorizontalTab,
            0x0a => C0ControlKind::LineFeed,
            0x0b => C0ControlKind::VerticalTab,
            0x0c => C0ControlKind::FormFeed,
            0x0d => C0ControlKind::CarriageReturn,
            0x1b => C0ControlKind::Escape,
            0x7f => C0ControlKind::Delete,
            _ => C0ControlKind::Other,
        };

        Self { byte, kind }
    }

    #[must_use]
    pub const fn byte(self) -> u8 {
        self.byte
    }

    #[must_use]
    pub const fn kind(self) -> C0ControlKind {
        self.kind
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum C0ControlKind {
    Null,
    Bell,
    Backspace,
    HorizontalTab,
    LineFeed,
    VerticalTab,
    FormFeed,
    CarriageReturn,
    Escape,
    Delete,
    Other,
}

/// Finalized ESC sequence before semantic dispatch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EscapeSequence {
    intermediates: Vec<u8>,
    ignored: bool,
    final_byte: u8,
}

impl EscapeSequence {
    #[must_use]
    pub fn new(intermediates: &[u8], ignored: bool, final_byte: u8) -> Self {
        Self {
            intermediates: intermediates.to_vec(),
            ignored,
            final_byte,
        }
    }

    #[must_use]
    pub fn intermediates(&self) -> &[u8] {
        &self.intermediates
    }

    #[must_use]
    pub const fn ignored(&self) -> bool {
        self.ignored
    }

    #[must_use]
    pub const fn final_byte(&self) -> u8 {
        self.final_byte
    }
}

/// Finalized CSI sequence with parameter and subparameter boundaries intact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsiSequence {
    params: Vec<CsiParam>,
    intermediates: Vec<u8>,
    ignored: bool,
    action: char,
}

impl CsiSequence {
    #[must_use]
    pub fn new(params: Vec<CsiParam>, intermediates: &[u8], ignored: bool, action: char) -> Self {
        Self {
            params,
            intermediates: intermediates.to_vec(),
            ignored,
            action,
        }
    }

    #[must_use]
    pub fn params(&self) -> &[CsiParam] {
        &self.params
    }

    #[must_use]
    pub fn intermediates(&self) -> &[u8] {
        &self.intermediates
    }

    #[must_use]
    pub const fn ignored(&self) -> bool {
        self.ignored
    }

    #[must_use]
    pub const fn action(&self) -> char {
        self.action
    }
}

/// One CSI parameter, including colon-separated subparameters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsiParam {
    subparameters: Vec<u16>,
}

impl CsiParam {
    #[must_use]
    pub fn new(subparameters: impl Into<Vec<u16>>) -> Self {
        Self {
            subparameters: subparameters.into(),
        }
    }

    #[must_use]
    pub fn subparameters(&self) -> &[u16] {
        &self.subparameters
    }
}

/// Operating system command payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OscCommand {
    payload: Payload,
    bell_terminated: bool,
}

impl OscCommand {
    #[must_use]
    pub const fn from_payload(payload: Payload, bell_terminated: bool) -> Self {
        Self {
            payload,
            bell_terminated,
        }
    }

    #[must_use]
    pub fn from_parts<'a>(
        parts: impl IntoIterator<Item = &'a [u8]>,
        bell_terminated: bool,
        limit: usize,
    ) -> Self {
        Self {
            payload: Payload::bounded_join(parts, b';', limit),
            bell_terminated,
        }
    }

    #[must_use]
    pub const fn payload(&self) -> &Payload {
        &self.payload
    }

    #[must_use]
    pub const fn bell_terminated(&self) -> bool {
        self.bell_terminated
    }
}

/// Device control string payload with its introducer metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DcsCommand {
    params: Vec<CsiParam>,
    intermediates: Vec<u8>,
    ignored: bool,
    action: char,
    payload: Payload,
}

impl DcsCommand {
    #[must_use]
    pub fn new(
        params: Vec<CsiParam>,
        intermediates: &[u8],
        ignored: bool,
        action: char,
        payload: Payload,
    ) -> Self {
        Self {
            params,
            intermediates: intermediates.to_vec(),
            ignored,
            action,
            payload,
        }
    }

    #[must_use]
    pub fn params(&self) -> &[CsiParam] {
        &self.params
    }

    #[must_use]
    pub fn intermediates(&self) -> &[u8] {
        &self.intermediates
    }

    #[must_use]
    pub const fn ignored(&self) -> bool {
        self.ignored
    }

    #[must_use]
    pub const fn action(&self) -> char {
        self.action
    }

    #[must_use]
    pub const fn payload(&self) -> &Payload {
        &self.payload
    }
}

/// APC or PM string control payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StringControl {
    kind: StringControlKind,
    payload: Payload,
}

impl StringControl {
    #[must_use]
    pub const fn new(kind: StringControlKind, payload: Payload) -> Self {
        Self { kind, payload }
    }

    #[must_use]
    pub const fn kind(&self) -> StringControlKind {
        self.kind
    }

    #[must_use]
    pub const fn payload(&self) -> &Payload {
        &self.payload
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringControlKind {
    ApplicationProgramCommand,
    PrivacyMessage,
}

/// Bounded string-control payload plus limit diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Payload {
    bytes: Vec<u8>,
    original_len: usize,
    status: PayloadStatus,
}

impl Payload {
    #[must_use]
    pub fn bounded(bytes: &[u8], limit: usize) -> Self {
        let retained_len = bytes.len().min(limit);
        Self::from_limited_bytes(bytes[..retained_len].to_vec(), bytes.len(), limit)
    }

    #[must_use]
    pub fn bounded_join<'a>(
        parts: impl IntoIterator<Item = &'a [u8]>,
        separator: u8,
        limit: usize,
    ) -> Self {
        let mut bytes = Vec::new();
        let mut original_len = 0usize;

        for (index, part) in parts.into_iter().enumerate() {
            if index > 0 {
                original_len += 1;
                if bytes.len() < limit {
                    bytes.push(separator);
                }
            }

            original_len += part.len();
            let remaining = limit.saturating_sub(bytes.len());
            bytes.extend_from_slice(&part[..part.len().min(remaining)]);
        }

        Self::from_limited_bytes(bytes, original_len, limit)
    }

    #[must_use]
    pub fn from_limited_bytes(mut bytes: Vec<u8>, original_len: usize, limit: usize) -> Self {
        if bytes.len() > limit {
            bytes.truncate(limit);
        }

        let status = if original_len > bytes.len() || original_len > limit {
            PayloadStatus::Truncated {
                original_len,
                retained_len: bytes.len(),
                limit,
            }
        } else {
            PayloadStatus::Complete
        };

        Self {
            bytes,
            original_len,
            status,
        }
    }

    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[must_use]
    pub const fn original_len(&self) -> usize {
        self.original_len
    }

    #[must_use]
    pub const fn status(&self) -> &PayloadStatus {
        &self.status
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PayloadStatus {
    Complete,
    Truncated {
        original_len: usize,
        retained_len: usize,
        limit: usize,
    },
    Ignored {
        original_len: usize,
        limit: usize,
    },
}

/// Unsupported input that reached Hera's protocol boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsupportedSequence {
    kind: UnsupportedSequenceKind,
    diagnostic: String,
}

impl UnsupportedSequence {
    #[must_use]
    pub fn new(kind: UnsupportedSequenceKind, diagnostic: impl Into<String>) -> Self {
        Self {
            kind,
            diagnostic: diagnostic.into(),
        }
    }

    #[must_use]
    pub const fn kind(&self) -> UnsupportedSequenceKind {
        self.kind
    }

    #[must_use]
    pub fn diagnostic(&self) -> &str {
        &self.diagnostic
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnsupportedSequenceKind {
    PayloadLimitExceeded,
    ParserIgnored,
    UnterminatedDcs,
    Other,
}

#[cfg(test)]
mod tests {
    use super::{CsiParam, M1_PAYLOAD_LIMIT_BYTES, Payload, PayloadStatus};

    #[test]
    fn csi_param_preserves_subparameter_boundaries() {
        let params = [CsiParam::new([38, 2, 255, 128, 64]), CsiParam::new([0])];

        assert_eq!(params[0].subparameters(), &[38, 2, 255, 128, 64]);
        assert_eq!(params[1].subparameters(), &[0]);
    }

    #[test]
    fn oversized_payload_is_truncated_with_diagnostics() {
        let bytes = vec![b'x'; M1_PAYLOAD_LIMIT_BYTES + 17];
        let payload = Payload::bounded(&bytes, M1_PAYLOAD_LIMIT_BYTES);

        assert_eq!(payload.bytes().len(), M1_PAYLOAD_LIMIT_BYTES);
        assert_eq!(payload.original_len(), M1_PAYLOAD_LIMIT_BYTES + 17);
        assert_eq!(
            payload.status(),
            &PayloadStatus::Truncated {
                original_len: M1_PAYLOAD_LIMIT_BYTES + 17,
                retained_len: M1_PAYLOAD_LIMIT_BYTES,
                limit: M1_PAYLOAD_LIMIT_BYTES,
            }
        );
    }

    #[test]
    fn retained_payload_shorter_than_original_is_truncated() {
        let payload = Payload::from_limited_bytes(vec![b'x'; 4], 9, M1_PAYLOAD_LIMIT_BYTES);

        assert_eq!(
            payload.status(),
            &PayloadStatus::Truncated {
                original_len: 9,
                retained_len: 4,
                limit: M1_PAYLOAD_LIMIT_BYTES,
            }
        );
    }
}
