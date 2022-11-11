pub use textparse_derive::Span;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Position(usize);

impl Position {
    pub const fn new(offset: usize) -> Self {
        Self(offset)
    }

    pub const fn get(self) -> usize {
        self.0
    }
}

pub trait Span {
    fn start_position(&self) -> Position;
    fn end_position(&self) -> Position;

    fn is_empty(&self) -> bool {
        self.start_position().get() >= self.end_position().get()
    }

    fn utf8_len(&self) -> usize {
        if self.is_empty() {
            0
        } else {
            self.end_position().get() - self.start_position().get()
        }
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
