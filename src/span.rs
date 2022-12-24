pub use textparse_derive::Span;

/// Position (offset) in a text.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Position(usize);

impl Position {
    /// Makes a new [`Position`] instance.
    pub const fn new(offset: usize) -> Self {
        Self(offset)
    }

    /// Gets the offset.
    pub const fn get(self) -> usize {
        self.0
    }

    /// Returns the line and column numbers at where this position is located in the given text.
    pub fn line_and_column(self, text: &str) -> (usize, usize) {
        let offset = self.0;
        let mut line = 0;
        let mut column = 1;
        for c in text[..offset].chars() {
            if c == '\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
        }
        (line, column)
    }
}

/// This trait allows for representing a parsed item that has start and end positions in a text.
pub trait Span {
    /// Returns the start position of this item.
    fn start_position(&self) -> Position;

    /// Returns the end position of this item.
    fn end_position(&self) -> Position;

    /// Returns `true` if the span is empty, otherwise `false`.
    fn is_empty(&self) -> bool {
        self.start_position().get() >= self.end_position().get()
    }

    /// Returns the length of this span.
    fn len(&self) -> usize {
        if self.is_empty() {
            0
        } else {
            self.end_position().get() - self.start_position().get()
        }
    }

    /// Returns the text representation of this item.
    fn text<'a>(&self, text: &'a str) -> &'a str {
        &text[self.start_position().get()..self.end_position().get()]
    }
}

impl Span for Position {
    fn start_position(&self) -> Position {
        *self
    }

    fn end_position(&self) -> Position {
        *self
    }
}

impl Span for std::ops::Range<Position> {
    fn start_position(&self) -> Position {
        self.start
    }

    fn end_position(&self) -> Position {
        self.end
    }
}

impl<T: Span> Span for Box<T> {
    fn start_position(&self) -> Position {
        (**self).start_position()
    }

    fn end_position(&self) -> Position {
        (**self).end_position()
    }
}

impl<T: Span> Span for &T {
    fn start_position(&self) -> Position {
        (**self).start_position()
    }

    fn end_position(&self) -> Position {
        (**self).end_position()
    }
}

impl<T0: Span, T1: Span> Span for (T0, T1) {
    fn start_position(&self) -> Position {
        self.0.start_position()
    }

    fn end_position(&self) -> Position {
        self.1.end_position()
    }
}

impl<T0: Span, T1: Span, T2: Span> Span for (T0, T1, T2) {
    fn start_position(&self) -> Position {
        self.0.start_position()
    }

    fn end_position(&self) -> Position {
        self.2.end_position()
    }
}

impl<T0: Span, T1: Span, T2: Span, T3: Span> Span for (T0, T1, T2, T3) {
    fn start_position(&self) -> Position {
        self.0.start_position()
    }

    fn end_position(&self) -> Position {
        self.3.end_position()
    }
}

impl<T0: Span, T1: Span, T2: Span, T3: Span, T4: Span> Span for (T0, T1, T2, T3, T4) {
    fn start_position(&self) -> Position {
        self.0.start_position()
    }

    fn end_position(&self) -> Position {
        self.4.end_position()
    }
}

impl<T0: Span, T1: Span, T2: Span, T3: Span, T4: Span, T5: Span> Span for (T0, T1, T2, T3, T4, T5) {
    fn start_position(&self) -> Position {
        self.0.start_position()
    }

    fn end_position(&self) -> Position {
        self.5.end_position()
    }
}
