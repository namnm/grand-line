mod intl;
pub use intl::*;

// Minimal lexer state for scanning Rhai source in context-aware mode.
// Tracks whether the current position is inside a string literal or comment.
pub struct CodeScanner<'a> {
    pub bytes: &'a [u8],
    pub pos: usize,
    string_delim: Option<u8>, // b'"' | b'\'' | b'`'
    in_line_comment: bool,
    in_block_comment: bool,
}

impl<'a> CodeScanner<'a> {
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            pos: 0,
            string_delim: None,
            in_line_comment: false,
            in_block_comment: false,
        }
    }

    pub const fn done(&self) -> bool {
        self.pos >= self.bytes.len()
    }

    // True when the current position is plain code (not inside a string or comment).
    pub const fn in_code(&self) -> bool {
        self.string_delim.is_none() && !self.in_line_comment && !self.in_block_comment
    }

    pub fn step(&mut self) {
        let Some(b) = self.bytes.get(self.pos).copied() else {
            return;
        };

        if self.in_line_comment {
            if b == b'\n' {
                self.in_line_comment = false;
            }
            self.pos += 1;
        } else if self.in_block_comment {
            if b == b'*' && self.bytes.get(self.pos + 1) == Some(&b'/') {
                self.in_block_comment = false;
                self.pos += 1;
            }
            self.pos += 1;
        } else if let Some(delim) = self.string_delim {
            if b == b'\\' && self.pos + 1 < self.bytes.len() {
                self.pos += 2;
            } else {
                if b == delim {
                    self.string_delim = None;
                }
                self.pos += 1;
            }
        } else {
            match b {
                b'"' | b'\'' | b'`' => {
                    self.string_delim = Some(b);
                    self.pos += 1;
                }
                b'/' if self.bytes.get(self.pos + 1) == Some(&b'/') => {
                    self.in_line_comment = true;
                    self.pos += 2;
                }
                b'/' if self.bytes.get(self.pos + 1) == Some(&b'*') => {
                    self.in_block_comment = true;
                    self.pos += 2;
                }
                _ => {
                    self.pos += 1;
                }
            }
        }
    }
}
