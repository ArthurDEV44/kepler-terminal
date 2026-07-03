use std::fmt;
use std::io::{Read, Write};

use crate::{PtyError, PtyExit, PtySessionConfig, PtySize};

pub const M2_MAX_WRITE_CHUNK_BYTES: usize = 64 * 1024;

pub trait PtyBackend {
    fn spawn(&self, config: &PtySessionConfig) -> Result<PtySession, PtyError>;
}

pub trait PtyChild: fmt::Debug + Send {
    fn try_wait(&mut self) -> Result<Option<PtyExit>, PtyError>;

    fn wait(&mut self) -> Result<PtyExit, PtyError>;

    fn process_id(&self) -> Option<u32>;

    fn kill(&mut self) -> Result<(), PtyError>;

    fn clone_killer(&self) -> Box<dyn PtyChildKiller>;
}

pub trait PtyChildKiller: fmt::Debug + Send {
    fn kill(&mut self) -> Result<(), PtyError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PtyPlatformMetadata {
    backend: &'static str,
    process_id: Option<u32>,
    size: PtySize,
}

impl PtyPlatformMetadata {
    #[must_use]
    pub const fn new(backend: &'static str, process_id: Option<u32>, size: PtySize) -> Self {
        Self {
            backend,
            process_id,
            size,
        }
    }

    #[must_use]
    pub const fn backend(&self) -> &'static str {
        self.backend
    }

    #[must_use]
    pub const fn process_id(&self) -> Option<u32> {
        self.process_id
    }

    #[must_use]
    pub const fn size(&self) -> PtySize {
        self.size
    }
}

pub struct PtySession {
    reader: Option<Box<dyn Read + Send>>,
    writer: Option<Box<dyn Write + Send>>,
    child: Option<Box<dyn PtyChild>>,
    metadata: PtyPlatformMetadata,
    backend: Box<dyn PtySessionBackend>,
}

impl PtySession {
    pub(crate) fn new(
        reader: Box<dyn Read + Send>,
        writer: Box<dyn Write + Send>,
        child: Box<dyn PtyChild>,
        metadata: PtyPlatformMetadata,
        backend: Box<dyn PtySessionBackend>,
    ) -> Self {
        Self {
            reader: Some(reader),
            writer: Some(writer),
            child: Some(child),
            metadata,
            backend,
        }
    }

    pub fn reader(&mut self) -> Option<&mut (dyn Read + Send + '_)> {
        self.reader
            .as_mut()
            .map(|reader| reader.as_mut() as &mut (dyn Read + Send + '_))
    }

    pub fn take_reader(&mut self) -> Option<Box<dyn Read + Send>> {
        self.reader.take()
    }

    pub fn writer(&mut self) -> Option<&mut (dyn Write + Send + '_)> {
        self.writer
            .as_mut()
            .map(|writer| writer.as_mut() as &mut (dyn Write + Send + '_))
    }

    pub fn take_writer(&mut self) -> Option<Box<dyn Write + Send>> {
        self.writer.take()
    }

    pub fn close_input(&mut self) {
        self.writer = None;
    }

    pub fn child(&mut self) -> Option<&mut (dyn PtyChild + '_)> {
        self.child
            .as_mut()
            .map(|child| child.as_mut() as &mut (dyn PtyChild + '_))
    }

    pub fn take_child(&mut self) -> Option<Box<dyn PtyChild>> {
        self.child.take()
    }

    #[must_use]
    pub const fn metadata(&self) -> &PtyPlatformMetadata {
        &self.metadata
    }

    pub fn resize(&self, size: PtySize) -> Result<(), PtyError> {
        self.backend.resize(size)
    }
}

pub(crate) trait PtySessionBackend: Send {
    fn resize(&self, size: PtySize) -> Result<(), PtyError>;
}

impl fmt::Debug for PtySession {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PtySession")
            .field("metadata", &self.metadata)
            .field("reader_available", &self.reader.is_some())
            .field("writer_available", &self.writer.is_some())
            .field("child_available", &self.child.is_some())
            .finish_non_exhaustive()
    }
}
