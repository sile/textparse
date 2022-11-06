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
        !(self.start_position().get() < self.end_position().get())
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
