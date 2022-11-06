use std::marker::PhantomData;

use crate::{Parse, ParseError, ParseResult, Parser, Position, Span};

#[derive(Debug, Clone, Span)]
pub struct Empty {
    position: Position,
}

impl Empty {
    pub const fn new(position: Position) -> Self {
        Self { position }
    }
}

impl Parse for Empty {
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
pub struct Maybe<T>(Either<T, Empty>);

impl<T> Maybe<T> {
    pub const fn some(value: T) -> Self {
        Self(Either::A(value))
    }

    pub const fn none(position: Position) -> Self {
        Self(Either::B(Empty::new(position)))
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
}

#[derive(Debug)]
pub struct Until<T> {
    start_position: Position,
    end_position: Position,
    _phantom: PhantomData<T>,
}

impl<T> Clone for Until<T> {
    fn clone(&self) -> Self {
        Self {
            start_position: self.start_position,
            end_position: self.end_position,
            _phantom: self._phantom,
        }
    }
}

impl<T> Span for Until<T> {
    fn start_position(&self) -> Position {
        self.start_position
    }

    fn end_position(&self) -> Position {
        self.end_position
    }
}

impl<T: Parse> Parse for Until<T> {
    fn parse(parser: &mut Parser) -> ParseResult<Self> {
        let start_position = parser.current_position();
        let mut end_position = parser.current_position();
        while parser.parse::<T>().is_err() {
            if let Some(c) = parser.remaining_text().chars().next() {
                parser.consume_bytes(c.len_utf8());
                end_position = parser.current_position();
            } else {
                return Err(ParseError);
            }
        }
        parser.set_current_position(end_position);
        Ok(Self {
            start_position,
            end_position,
            _phantom: PhantomData,
        })
    }
}

#[derive(Debug, Clone, Span)]
pub struct While<T> {
    start_position: Position,
    _phantom: PhantomData<T>,
    end_position: Position,
}

impl<T: Parse> Parse for While<T> {
    fn parse(parser: &mut Parser) -> ParseResult<Self> {
        let start_position = parser.current_position();
        while parser.parse::<T>().is_ok() {}
        let end_position = parser.current_position();
        Ok(Self {
            start_position,
            end_position,
            _phantom: PhantomData,
        })
    }
}

#[derive(Debug, Clone, Span)]
pub struct Whitespace {
    start_position: Position,
    end_position: Position,
}

impl Parse for Whitespace {
    fn parse(parser: &mut Parser) -> ParseResult<Self> {
        if parser
            .remaining_text()
            .chars()
            .next()
            .map_or(false, |c| c.is_ascii_whitespace())
        {
            let (start_position, end_position) = parser.consume_chars(1);
            Ok(Self {
                start_position,
                end_position,
            })
        } else {
            Err(ParseError)
        }
    }
}

#[derive(Debug, Clone, Span, Parse)]
pub struct Whitespaces(While<Whitespace>);

#[derive(Debug, Clone, Span)]
pub struct SkipWhitespaces(Empty);

impl Parse for SkipWhitespaces {
    fn parse(parser: &mut Parser) -> ParseResult<Self> {
        parser.parse::<Whitespaces>()?;
        parser.parse().map(Self)
    }
}

#[derive(Debug, Clone, Span)]
pub struct Char<const T: char> {
    start_position: Position,
    end_position: Position,
}

impl<const T: char> Parse for Char<T> {
    fn parse(parser: &mut Parser) -> ParseResult<Self> {
        if parser.remaining_text().starts_with(T) {
            let (start_position, end_position) = parser.consume_bytes(T.len_utf8());
            Ok(Self {
                start_position,
                end_position,
            })
        } else {
            Err(ParseError)
        }
    }

    fn name() -> Option<fn() -> String> {
        Some(|| format!("{:?}", T))
    }
}

#[derive(Debug)]
pub struct StartsWith<T> {
    start_position: Position,
    end_position: Position,
    _static_str: PhantomData<T>,
}

impl<T> Clone for StartsWith<T> {
    fn clone(&self) -> Self {
        Self {
            start_position: self.start_position,
            end_position: self.end_position,
            _static_str: self._static_str,
        }
    }
}

impl<T> Span for StartsWith<T> {
    fn start_position(&self) -> Position {
        self.start_position
    }

    fn end_position(&self) -> Position {
        self.end_position
    }
}

impl<T: StaticStr> Parse for StartsWith<T> {
    fn parse(parser: &mut Parser) -> ParseResult<Self> {
        if parser.remaining_text().starts_with(T::static_str()) {
            let (start_position, end_position) = parser.consume_bytes(T::static_str().len());
            Ok(Self {
                start_position,
                end_position,
                _static_str: PhantomData,
            })
        } else {
            Err(ParseError)
        }
    }

    fn name() -> Option<fn() -> String> {
        Some(|| format!("{:?}", T::static_str()))
    }
}

pub trait StaticStr: 'static {
    fn static_str() -> &'static str;
}

#[derive(Debug, Clone)]
pub struct NonEmptyItems<Item, Delimiter> {
    items: Vec<Item>,
    delimiters: Vec<Delimiter>,
}

impl<Item, Delimiter> NonEmptyItems<Item, Delimiter> {
    pub fn items(&self) -> &[Item] {
        &self.items
    }

    pub fn delimiters(&self) -> &[Delimiter] {
        &self.delimiters
    }
}

impl<Item: Span, Delimiter: Span> Span for NonEmptyItems<Item, Delimiter> {
    fn start_position(&self) -> Position {
        self.items[0].start_position()
    }

    fn end_position(&self) -> Position {
        self.items[self.items.len() - 1].end_position()
    }
}

impl<Item: Parse, Delimiter: Parse> Parse for NonEmptyItems<Item, Delimiter> {
    fn parse(parser: &mut Parser) -> ParseResult<Self> {
        let mut items = vec![parser.parse::<Item>()?];
        let mut delimiters = Vec::new();
        while let Ok(delimiter) = parser.parse::<Delimiter>() {
            delimiters.push(delimiter);
            items.push(parser.parse()?);
        }
        Ok(Self { items, delimiters })
    }
}

#[derive(Debug, Clone, Span, Parse)]
pub struct Items<Item, Delimiter>(Maybe<NonEmptyItems<Item, Delimiter>>);

impl<Item, Delimiter> Items<Item, Delimiter> {
    pub fn items(&self) -> &[Item] {
        self.0.get().map_or(&[], |t| t.items())
    }

    pub fn delimiters(&self) -> &[Delimiter] {
        self.0.get().map_or(&[], |t| t.delimiters())
    }
}

#[derive(Debug, Clone, Span)]
pub struct NonEmpty<T>(T);

impl<T> NonEmpty<T> {
    pub fn get(&self) -> &T {
        &self.0
    }
}

impl<T: Parse> Parse for NonEmpty<T> {
    fn parse(parser: &mut Parser) -> ParseResult<Self> {
        let item: T = parser.parse()?;
        if item.is_empty() {
            Err(ParseError)
        } else {
            Ok(Self(item))
        }
    }
}

#[derive(Debug, Clone, Span)]
pub struct Eos {
    position: Position,
}

impl Parse for Eos {
    fn parse(parser: &mut Parser) -> ParseResult<Self> {
        if parser.is_eos() {
            Ok(Self {
                position: parser.current_position(),
            })
        } else {
            Err(ParseError)
        }
    }

    fn name() -> Option<fn() -> String> {
        Some(|| "EOS".to_owned())
    }
}
