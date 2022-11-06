use std::marker::PhantomData;

use crate::{Parse, ParseError, ParseResult, Parser, Position, Span};

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

    fn name() -> String {
        T::name()
    }
}

#[derive(Debug, Clone, Span)]
pub struct Char<const T: char> {
    start_position: Position,
    end_position: Position,
}

impl<const T: char> Parse for Char<T> {
    fn parse(parser: &mut Parser) -> ParseResult<Self> {
        if parser.remaining_text().chars().next() == Some(T) {
            let (start_position, end_position) = parser.consume_bytes(T.len_utf8());
            Ok(Self {
                start_position,
                end_position,
            })
        } else {
            Err(ParseError)
        }
    }

    fn name() -> String {
        format!("{:?}", T)
    }
}

#[derive(Debug)]
pub struct While<P> {
    start_position: Position,
    end_position: Position,
    _predicate: PhantomData<P>,
}

impl<P> Clone for While<P> {
    fn clone(&self) -> Self {
        Self {
            start_position: self.start_position.clone(),
            end_position: self.end_position.clone(),
            _predicate: self._predicate,
        }
    }
}

impl<P> Span for While<P> {
    fn start_position(&self) -> Position {
        self.start_position
    }

    fn end_position(&self) -> Position {
        self.end_position
    }
}

impl<P: Predicate> Parse for While<P> {
    fn parse(parser: &mut Parser) -> ParseResult<Self> {
        let n = parser
            .remaining_text()
            .chars()
            .take_while(|c| P::is(*c))
            .map(|c| c.len_utf8())
            .sum();
        let (start_position, end_position) = parser.consume_bytes(n);
        Ok(Self {
            start_position,
            end_position,
            _predicate: PhantomData,
        })
    }
}

pub trait Predicate: 'static {
    fn is(c: char) -> bool;
}

#[derive(Debug)]
struct IsWhiteSpace;

impl Predicate for IsWhiteSpace {
    fn is(c: char) -> bool {
        c.is_ascii_whitespace()
    }
}

#[derive(Debug, Clone, Span, Parse)]
pub struct Whitespaces(While<IsWhiteSpace>);

#[derive(Debug, Clone, Span)]
pub struct SkipWhitespaces(Null);

impl Parse for SkipWhitespaces {
    fn parse(parser: &mut Parser) -> ParseResult<Self> {
        parser.parse::<Whitespaces>()?;
        parser.parse().map(Self)
    }
}
