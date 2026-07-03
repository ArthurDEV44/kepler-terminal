use crate::PtyError;

pub const M2_MAX_PTY_COLUMNS: u16 = 4096;
pub const M2_MAX_PTY_ROWS: u16 = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PtySize {
    columns: u16,
    rows: u16,
    pixel_width: u16,
    pixel_height: u16,
}

impl PtySize {
    pub fn new(columns: u16, rows: u16) -> Result<Self, PtyError> {
        Self::with_cell_pixels(columns, rows, 0, 0)
    }

    pub fn with_cell_pixels(
        columns: u16,
        rows: u16,
        pixel_width: u16,
        pixel_height: u16,
    ) -> Result<Self, PtyError> {
        if columns == 0 || rows == 0 || columns > M2_MAX_PTY_COLUMNS || rows > M2_MAX_PTY_ROWS {
            return Err(PtyError::InvalidSize {
                columns,
                rows,
                max_columns: M2_MAX_PTY_COLUMNS,
                max_rows: M2_MAX_PTY_ROWS,
            });
        }

        Ok(Self {
            columns,
            rows,
            pixel_width,
            pixel_height,
        })
    }

    #[must_use]
    pub const fn columns(self) -> u16 {
        self.columns
    }

    #[must_use]
    pub const fn rows(self) -> u16 {
        self.rows
    }

    #[must_use]
    pub const fn pixel_width(self) -> u16 {
        self.pixel_width
    }

    #[must_use]
    pub const fn pixel_height(self) -> u16 {
        self.pixel_height
    }

    pub(crate) const fn to_portable(self) -> portable_pty::PtySize {
        portable_pty::PtySize {
            rows: self.rows,
            cols: self.columns,
            pixel_width: self.pixel_width,
            pixel_height: self.pixel_height,
        }
    }
}

impl Default for PtySize {
    fn default() -> Self {
        Self {
            columns: 80,
            rows: 24,
            pixel_width: 0,
            pixel_height: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{M2_MAX_PTY_COLUMNS, PtySize};
    use crate::{PtyError, PtyErrorKind};

    #[test]
    fn zero_dimensions_return_typed_validation_error() {
        let error = PtySize::new(0, 24).expect_err("zero columns must fail");

        assert_eq!(error.kind(), PtyErrorKind::InvalidSize);
        assert!(matches!(
            error,
            PtyError::InvalidSize {
                columns: 0,
                rows: 24,
                ..
            }
        ));
    }

    #[test]
    fn dimensions_above_m2_limit_return_typed_validation_error() {
        let error =
            PtySize::new(M2_MAX_PTY_COLUMNS + 1, 24).expect_err("oversized columns must fail");

        assert_eq!(error.kind(), PtyErrorKind::InvalidSize);
    }
}
