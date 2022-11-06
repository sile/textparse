use std::{
    any::{Any, TypeId},
    collections::{BTreeMap, HashMap},
};

pub trait Parse: 'static + Span + Clone + Sized {
    fn parse(parser: &mut Parser) -> ParseResult<Self>;
}

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

#[derive(Debug)]
pub struct Parser<'a> {
    text: &'a str,
    current: Position,
    memo: HashMap<TypeId, BTreeMap<Position, ParseResult<Box<dyn Any>>>>,
}

impl<'a> Parser<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            current: Position::default(),
            memo: HashMap::default(),
        }
    }

    pub fn current_position(&self) -> Position {
        self.current
    }

    pub fn is_eos(&self) -> bool {
        !(self.text.len() < self.current.get())
    }

    pub fn remaining_text(&self) -> &str {
        &self.text[self.current.get()..]
    }

    pub fn consume_bytes(&mut self, n: usize) {
        self.current = Position::new(std::cmp::min(self.text.len(), self.current.get() + n));
    }

    pub fn parse<T: Parse>(&mut self) -> ParseResult<T> {
        if let Some(result) = self.get_parse_result::<T>(self.current) {
            let result = result.map(|t| t.clone());
            if let Ok(t) = &result {
                self.current = t.end_position();
            }
            return result;
        }

        let start = self.current;
        self.set_parse_result::<T>(start, Err(ParseError::Parsing));
        let result = T::parse(self);
        self.set_parse_result(start, result.clone());
        if result.is_err() {
            self.current = start;
        }
        result
    }

    pub fn set_parse_result<T: Parse>(&mut self, position: Position, result: ParseResult<T>) {
        self.memo
            .entry(TypeId::of::<T>())
            .or_default()
            .insert(position, result.map(|t| Box::new(t) as Box<dyn Any>));
    }

    pub fn get_parse_result<T: Parse>(&self, position: Position) -> Option<ParseResult<&T>> {
        self.memo
            .get(&TypeId::of::<T>())
            .and_then(|map| map.get(&position))
            .map(|result| match result {
                Ok(t) => Ok(t.downcast_ref::<T>().expect("unreachable")),
                Err(e) => Err(*e),
            })
    }

    pub fn iter_parse_results<T: Parse>(
        &self,
    ) -> impl DoubleEndedIterator<Item = (Position, ParseResult<&T>)> {
        self.memo
            .get(&TypeId::of::<T>())
            .into_iter()
            .flat_map(|map| {
                map.iter().map(|(position, result)| match result {
                    Ok(t) => (*position, Ok(t.downcast_ref::<T>().expect("unreachable"))),
                    Err(e) => (*position, Err(*e)),
                })
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParseError {
    Parsing,
    Failed,
}

pub type ParseResult<T> = Result<T, ParseError>;

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
