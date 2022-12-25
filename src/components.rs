//! Basic components.
use crate::{Parse, Parser, Position, Span};
use std::marker::PhantomData;

/// Empty item.
#[derive(Debug, Clone, Copy, Span)]
pub struct Empty {
    position: Position,
}

impl Parse for Empty {
    fn parse(parser: &mut Parser) -> Option<Self> {
        Some(Self {
            position: parser.current_position(),
        })
    }
}

/// Either `A` or `B`.
#[derive(Debug, Clone, Copy, Span, Parse)]
#[allow(missing_docs)]
pub enum Either<A, B> {
    A(A),
    B(B),
}

/// One of `A`, `B`, or `C`.
#[derive(Debug, Clone, Copy, Span, Parse)]
#[allow(missing_docs)]
pub enum OneOfThree<A, B, C> {
    A(A),
    B(B),
    C(C),
}

/// One of `A`, `B`, `C`, or `D`.
#[derive(Debug, Clone, Copy, Span, Parse)]
#[allow(missing_docs)]
pub enum OneOfFour<A, B, C, D> {
    A(A),
    B(B),
    C(C),
    D(D),
}

/// Optional item.
#[derive(Debug, Clone, Copy, Span)]
pub struct Maybe<T>(Either<T, Empty>);

impl<T> Maybe<T> {
    /// Returns `Some(T)` if the item exists, otherwise `None`.
    pub fn get(&self) -> Option<&T> {
        if let Either::A(t) = &self.0 {
            Some(t)
        } else {
            None
        }
    }
}

impl<T: Parse> Parse for Maybe<T> {
    fn parse(parser: &mut Parser) -> Option<Self> {
        parser.parse().map(Self)
    }
}

/// Indicating to continue parsing while `T::parse()` is succeeded.
#[derive(Debug, Span)]
pub struct While<T> {
    start_position: Position,
    _phantom: PhantomData<T>,
    end_position: Position,
}

impl<T: Parse> Parse for While<T> {
    fn parse(parser: &mut Parser) -> Option<Self> {
        let start_position = parser.current_position();
        while parser.parse::<T>().is_some() {}
        let end_position = parser.current_position();
        Some(Self {
            start_position,
            end_position,
            _phantom: PhantomData,
        })
    }
}

impl<T> Clone for While<T> {
    fn clone(&self) -> Self {
        Self {
            start_position: self.start_position,
            _phantom: self._phantom,
            end_position: self.end_position,
        }
    }
}

impl<T> Copy for While<T> {}

/// A whitespace (cf. [`char::is_ascii_whitespace()`]).
#[derive(Debug, Clone, Copy, Span)]
pub struct Whitespace {
    start_position: Position,
    end_position: Position,
}

impl Parse for Whitespace {
    fn parse(parser: &mut Parser) -> Option<Self> {
        let start_position = parser.current_position();
        parser
            .read_char()
            .filter(|c| c.is_ascii_whitespace())
            .map(|_| Self {
                start_position,
                end_position: parser.current_position(),
            })
    }
}

/// A character.
#[derive(Debug, Clone, Copy, Span)]
pub struct AnyChar {
    start_position: Position,
    value: char,
    end_position: Position,
}

impl AnyChar {
    /// Returns the character value.
    pub fn get(&self) -> char {
        self.value
    }
}

impl Parse for AnyChar {
    fn parse(parser: &mut Parser) -> Option<Self> {
        let start_position = parser.current_position();
        let value = parser.read_char()?;
        let end_position = parser.current_position();
        Some(Self {
            start_position,
            value,
            end_position,
        })
    }
}

/// A specific character.
#[derive(Debug, Clone, Copy, Span)]
pub struct Char<const T: char, const NAMED: bool = true> {
    start_position: Position,
    end_position: Position,
}

impl<const T: char, const NAMED: bool> Parse for Char<T, NAMED> {
    fn parse(parser: &mut Parser) -> Option<Self> {
        let start_position = parser.current_position();
        parser.read_char().filter(|c| *c == T).map(|_| Self {
            start_position,
            end_position: parser.current_position(),
        })
    }

    fn name() -> Option<fn() -> String> {
        if NAMED {
            Some(|| format!("{T:?}"))
        } else {
            None
        }
    }
}

/// A specified string (characters).
#[derive(Debug, Clone, Copy, Span)]
pub struct Str<
    const C0: char = '\0',
    const C1: char = '\0',
    const C2: char = '\0',
    const C3: char = '\0',
    const C4: char = '\0',
    const C5: char = '\0',
    const C6: char = '\0',
    const C7: char = '\0',
    const C8: char = '\0',
    const C9: char = '\0',
> {
    start_position: Position,
    end_position: Position,
}

impl<
        const C0: char,
        const C1: char,
        const C2: char,
        const C3: char,
        const C4: char,
        const C5: char,
        const C6: char,
        const C7: char,
        const C8: char,
        const C9: char,
    > Parse for Str<C0, C1, C2, C3, C4, C5, C6, C7, C8, C9>
{
    fn parse(parser: &mut Parser) -> Option<Self> {
        let start_position = parser.current_position();
        for c in [C0, C1, C2, C3, C4, C5, C6, C7, C8, C9] {
            if c == '\0' {
                break;
            }
            if parser.read_char() != Some(c) {
                return None;
            }
        }
        let end_position = parser.current_position();
        Some(Self {
            start_position,
            end_position,
        })
    }

    fn name() -> Option<fn() -> String> {
        Some(|| {
            let mut s = String::new();
            for c in [C0, C1, C2, C3, C4, C5, C6, C7, C8, C9] {
                if c == '\0' {
                    break;
                }
                s.push(c);
            }
            s
        })
    }
}

#[derive(Debug, Clone)]
struct NonEmptyItems<Item, Delimiter> {
    items: Vec<Item>,
    delimiters: Vec<Delimiter>,
}

impl<Item, Delimiter> NonEmptyItems<Item, Delimiter> {
    fn items(&self) -> &[Item] {
        &self.items
    }

    fn delimiters(&self) -> &[Delimiter] {
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
    fn parse(parser: &mut Parser) -> Option<Self> {
        let mut items = vec![parser.parse::<Item>()?];
        let mut delimiters = Vec::new();
        while let Some(delimiter) = parser.parse::<Delimiter>() {
            delimiters.push(delimiter);
            items.push(parser.parse()?);
        }
        Some(Self { items, delimiters })
    }
}

/// Variable length items split by delimiters.
#[derive(Debug, Clone, Span, Parse)]
pub struct Items<Item, Delimiter>(Maybe<NonEmptyItems<Item, Delimiter>>);

impl<Item, Delimiter> Items<Item, Delimiter> {
    /// Returns items.
    pub fn items(&self) -> &[Item] {
        self.0.get().map_or(&[], |t| t.items())
    }

    /// Returns delimiters.
    pub fn delimiters(&self) -> &[Delimiter] {
        self.0.get().map_or(&[], |t| t.delimiters())
    }
}

/// Non-empty item.
#[derive(Debug, Clone, Copy, Span)]
pub struct NonEmpty<T>(T);

impl<T> NonEmpty<T> {
    /// Returns the item.
    pub fn get(&self) -> &T {
        &self.0
    }
}

impl<T: Parse> Parse for NonEmpty<T> {
    fn parse(parser: &mut Parser) -> Option<Self> {
        let item: T = parser.parse()?;
        if item.is_empty() {
            None
        } else {
            Some(Self(item))
        }
    }
}

/// End-Of-String.
#[derive(Debug, Clone, Copy, Span)]
pub struct Eos {
    position: Position,
}

impl Parse for Eos {
    fn parse(parser: &mut Parser) -> Option<Self> {
        if parser.is_eos() {
            Some(Self {
                position: parser.current_position(),
            })
        } else {
            None
        }
    }

    fn name() -> Option<fn() -> String> {
        Some(|| "EOS".to_owned())
    }
}

/// Not a specified item.
#[derive(Debug)]
pub struct Not<T> {
    position: Position,
    _item: PhantomData<T>,
}

impl<T> Clone for Not<T> {
    fn clone(&self) -> Self {
        Self {
            position: self.position,
            _item: self._item,
        }
    }
}

impl<T> Copy for Not<T> {}

impl<T> Span for Not<T> {
    fn start_position(&self) -> Position {
        self.position
    }

    fn end_position(&self) -> Position {
        self.position
    }
}

impl<T: Parse> Parse for Not<T> {
    fn parse(parser: &mut Parser) -> Option<Self> {
        if parser.parse::<T>().is_none() {
            let position = parser.current_position();
            Some(Self {
                position,
                _item: PhantomData,
            })
        } else {
            None
        }
    }

    fn name() -> Option<fn() -> String> {
        if T::name().is_none() {
            None
        } else {
            Some(|| format!("not {}", T::name().unwrap()()))
        }
    }
}

/// A digit.
#[derive(Debug, Clone, Copy, Span)]
pub struct Digit<const RADIX: u8 = 10> {
    start_position: Position,
    value: u8,
    end_position: Position,
}

impl<const RADIX: u8> Digit<RADIX> {
    /// Returns the digit value.
    pub const fn get(self) -> u8 {
        self.value
    }
}

impl<const RADIX: u8> Parse for Digit<RADIX> {
    fn parse(parser: &mut Parser) -> Option<Self> {
        let start_position = parser.current_position();
        let value = parser
            .read_char()
            .and_then(|c| c.to_digit(u32::from(RADIX)))? as u8;
        Some(Self {
            start_position,
            value,
            end_position: parser.current_position(),
        })
    }
}
