//! Renderer-neutral snapshot and damage model boundary for Hera M1.

#![forbid(unsafe_code)]

use std::mem;

/// Character used for deterministic empty cells in M1 snapshots.
pub const EMPTY_CELL_CHAR: char = ' ';

/// Active screen identity exposed to renderers and snapshot consumers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenIdentity {
    Primary,
    Alternate,
}

/// Stable row identity. Generation is reserved for future reflow and page reuse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RowHandle {
    id: u64,
    generation: u64,
}

impl RowHandle {
    #[must_use]
    pub const fn new(id: u64, generation: u64) -> Self {
        Self { id, generation }
    }

    #[must_use]
    pub const fn id(self) -> u64 {
        self.id
    }

    #[must_use]
    pub const fn generation(self) -> u64 {
        self.generation
    }
}

/// Cursor state in viewport coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorState {
    row: usize,
    column: usize,
    visible: bool,
}

impl CursorState {
    #[must_use]
    pub const fn new(row: usize, column: usize, visible: bool) -> Self {
        Self {
            row,
            column,
            visible,
        }
    }

    #[must_use]
    pub const fn row(self) -> usize {
        self.row
    }

    #[must_use]
    pub const fn column(self) -> usize {
        self.column
    }

    #[must_use]
    pub const fn visible(self) -> bool {
        self.visible
    }
}

/// Renderer-neutral color metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Indexed(u8),
    Rgb { red: u8, green: u8, blue: u8 },
}

/// Deterministic style defaults for snapshot cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellStyle {
    foreground: Option<Color>,
    background: Option<Color>,
    bold: bool,
    italic: bool,
    underline: bool,
    inverse: bool,
}

impl CellStyle {
    #[must_use]
    pub const fn new(
        foreground: Option<Color>,
        background: Option<Color>,
        bold: bool,
        italic: bool,
        underline: bool,
        inverse: bool,
    ) -> Self {
        Self {
            foreground,
            background,
            bold,
            italic,
            underline,
            inverse,
        }
    }

    #[must_use]
    pub const fn foreground(self) -> Option<Color> {
        self.foreground
    }

    #[must_use]
    pub const fn background(self) -> Option<Color> {
        self.background
    }

    #[must_use]
    pub const fn bold(self) -> bool {
        self.bold
    }

    #[must_use]
    pub const fn italic(self) -> bool {
        self.italic
    }

    #[must_use]
    pub const fn underline(self) -> bool {
        self.underline
    }

    #[must_use]
    pub const fn inverse(self) -> bool {
        self.inverse
    }
}

impl Default for CellStyle {
    fn default() -> Self {
        Self::new(None, None, false, false, false, false)
    }
}

/// Protocol family for an unsupported image placeholder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageProtocol {
    Kitty,
    Iterm2,
    Sixel,
    Unknown,
}

/// Safe image metadata. M1 snapshots never expose decoded image bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImagePlaceholder {
    protocol: ImageProtocol,
    id: Option<String>,
    byte_len: usize,
    diagnostic: String,
}

impl ImagePlaceholder {
    #[must_use]
    pub fn new(
        protocol: ImageProtocol,
        id: Option<String>,
        byte_len: usize,
        diagnostic: impl Into<String>,
    ) -> Self {
        Self {
            protocol,
            id,
            byte_len,
            diagnostic: diagnostic.into(),
        }
    }

    #[must_use]
    pub const fn protocol(&self) -> ImageProtocol {
        self.protocol
    }

    #[must_use]
    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    #[must_use]
    pub const fn byte_len(&self) -> usize {
        self.byte_len
    }

    #[must_use]
    pub fn diagnostic(&self) -> &str {
        &self.diagnostic
    }

    #[must_use]
    pub fn estimated_bytes(&self) -> usize {
        mem::size_of::<Self>()
            .saturating_add(self.id.as_ref().map_or(0, String::capacity))
            .saturating_add(self.diagnostic.capacity())
    }
}

/// One renderer-neutral viewport cell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderCell {
    ch: char,
    width: u8,
    style: CellStyle,
    image: Option<ImagePlaceholder>,
}

impl RenderCell {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            ch: EMPTY_CELL_CHAR,
            width: 1,
            style: CellStyle::default(),
            image: None,
        }
    }

    #[must_use]
    pub fn text(ch: char, width: u8, style: CellStyle) -> Self {
        Self {
            ch,
            width,
            style,
            image: None,
        }
    }

    #[must_use]
    pub fn image_placeholder(style: CellStyle, image: ImagePlaceholder) -> Self {
        Self {
            ch: EMPTY_CELL_CHAR,
            width: 1,
            style,
            image: Some(image),
        }
    }

    #[must_use]
    pub const fn ch(&self) -> char {
        self.ch
    }

    #[must_use]
    pub const fn width(&self) -> u8 {
        self.width
    }

    #[must_use]
    pub const fn style(&self) -> CellStyle {
        self.style
    }

    #[must_use]
    pub fn image(&self) -> Option<&ImagePlaceholder> {
        self.image.as_ref()
    }

    #[must_use]
    pub fn estimated_extra_bytes(&self) -> usize {
        self.image().map_or(0, ImagePlaceholder::estimated_bytes)
    }
}

impl Default for RenderCell {
    fn default() -> Self {
        Self::empty()
    }
}

/// One visible viewport row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewportRow {
    handle: RowHandle,
    cells: Vec<RenderCell>,
    wrapped: bool,
}

impl ViewportRow {
    #[must_use]
    pub fn new(handle: RowHandle, cells: Vec<RenderCell>, wrapped: bool) -> Self {
        Self {
            handle,
            cells,
            wrapped,
        }
    }

    #[must_use]
    pub const fn handle(&self) -> RowHandle {
        self.handle
    }

    #[must_use]
    pub fn cells(&self) -> &[RenderCell] {
        &self.cells
    }

    #[must_use]
    pub const fn wrapped(&self) -> bool {
        self.wrapped
    }
}

/// One scrollback row addressable by stable handle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScrollbackRow {
    handle: RowHandle,
    cells: Vec<RenderCell>,
    wrapped: bool,
}

impl ScrollbackRow {
    #[must_use]
    pub fn new(handle: RowHandle, cells: Vec<RenderCell>, wrapped: bool) -> Self {
        Self {
            handle,
            cells,
            wrapped,
        }
    }

    #[must_use]
    pub const fn handle(&self) -> RowHandle {
        self.handle
    }

    #[must_use]
    pub fn cells(&self) -> &[RenderCell] {
        &self.cells
    }

    #[must_use]
    pub const fn wrapped(&self) -> bool {
        self.wrapped
    }
}

/// Dirty row interval in viewport coordinates. End column is exclusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DamageRegion {
    row: usize,
    start_column: usize,
    end_column: usize,
}

impl DamageRegion {
    #[must_use]
    pub const fn new(row: usize, start_column: usize, end_column: usize) -> Self {
        Self {
            row,
            start_column,
            end_column,
        }
    }

    #[must_use]
    pub const fn row(self) -> usize {
        self.row
    }

    #[must_use]
    pub const fn start_column(self) -> usize {
        self.start_column
    }

    #[must_use]
    pub const fn end_column(self) -> usize {
        self.end_column
    }
}

/// Pull-based render snapshot for headless consumers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderSnapshot {
    columns: usize,
    rows: usize,
    active_screen: ScreenIdentity,
    cursor: CursorState,
    viewport_rows: Vec<ViewportRow>,
    scrollback_rows: Vec<ScrollbackRow>,
    damage: Vec<DamageRegion>,
}

impl RenderSnapshot {
    #[must_use]
    pub fn new(
        columns: usize,
        rows: usize,
        active_screen: ScreenIdentity,
        cursor: CursorState,
        viewport_rows: Vec<ViewportRow>,
        scrollback_rows: Vec<ScrollbackRow>,
        damage: Vec<DamageRegion>,
    ) -> Self {
        Self {
            columns,
            rows,
            active_screen,
            cursor,
            viewport_rows,
            scrollback_rows,
            damage,
        }
    }

    #[must_use]
    pub const fn columns(&self) -> usize {
        self.columns
    }

    #[must_use]
    pub const fn rows(&self) -> usize {
        self.rows
    }

    #[must_use]
    pub const fn active_screen(&self) -> ScreenIdentity {
        self.active_screen
    }

    #[must_use]
    pub const fn cursor(&self) -> CursorState {
        self.cursor
    }

    #[must_use]
    pub fn viewport_rows(&self) -> &[ViewportRow] {
        &self.viewport_rows
    }

    #[must_use]
    pub fn scrollback_rows(&self) -> &[ScrollbackRow] {
        &self.scrollback_rows
    }

    #[must_use]
    pub fn damage(&self) -> &[DamageRegion] {
        &self.damage
    }
}
