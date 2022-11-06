use crate::{Parse, ParseResult, Parser, Position, Span};

#[derive(Debug, Clone, Span)]
pub struct Null {
    position: Position,
}

impl Null {
    pub const fn new(position: Position) -> Self {
        Self { position }
    }
}

impl Parse for Null {
    fn parse(parser: &mut Parser) -> ParseResult<Self> {
        Ok(Self {
            position: parser.current_position(),
        })
    }
}

#[derive(Debug, Clone, Span, Parse)]
pub enum Either<A, B> {
    A(A),
    B(B),
}

#[derive(Debug, Clone, Span, Parse)]
pub enum OneOfThree<A, B, C> {
    A(A),
    B(B),
    C(C),
}

#[derive(Debug, Clone, Span, Parse)]
pub enum OneOfFour<A, B, C, D> {
    A(A),
    B(B),
    C(C),
    D(D),
}

#[derive(Debug, Clone, Span)]
pub struct Maybe<T>(Either<T, Null>);

impl<T> Maybe<T> {
    pub const fn some(value: T) -> Self {
        Self(Either::A(value))
    }

    pub const fn none(position: Position) -> Self {
        Self(Either::B(Null::new(position)))
    }

    pub fn get(&self) -> Option<&T> {
        if let Either::A(t) = &self.0 {
            Some(t)
        } else {
            None
        }
    }
}

impl<T: Parse> Parse for Maybe<T> {
    fn parse(parser: &mut Parser) -> ParseResult<Self> {
        parser.parse().map(Self)
    }

    fn name() -> &'static str {
        T::name()
    }
}
