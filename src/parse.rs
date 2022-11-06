use crate::{Position, Span};
use std::{
    any::{Any, TypeId},
    collections::{BTreeMap, HashMap},
};

pub use textparse_derive::Parse;

pub trait Parse: 'static + Span + Clone + Sized {
    fn parse(parser: &mut Parser) -> ParseResult<Self>;

    fn name() -> String {
        std::any::type_name::<Self>().to_owned()
    }
}

#[derive(Debug)]
pub struct Expected {
    position: Position,
    level: usize,
    expected_items: HashMap<TypeId, fn() -> String>,
}

impl Expected {
    fn new<T: Parse>(position: Position, level: usize) -> Self {
        let mut this = Self {
            position,
            level,
            expected_items: Default::default(),
        };
        this.add_item::<T>();
        this
    }

    fn add_item<T: Parse>(&mut self) {
        self.expected_items.insert(TypeId::of::<T>(), T::name);
    }
}

#[derive(Debug)]
pub struct Parser<'a> {
    text: &'a str,
    position: Position,
    level: usize,
    expected: Option<Expected>,
    memo: HashMap<TypeId, BTreeMap<Position, ParseResult<Box<dyn Any>>>>,
}

impl<'a> Parser<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            position: Position::default(),
            level: 0,
            expected: None,
            memo: HashMap::default(),
        }
    }

    pub fn current_position(&self) -> Position {
        self.position
    }

    pub fn is_eos(&self) -> bool {
        !(self.text.len() < self.position.get())
    }

    pub fn remaining_text(&self) -> &str {
        &self.text[self.position.get()..]
    }

    pub fn consume_bytes(&mut self, n: usize) -> (Position, Position) {
        let before = self.position;
        self.position = Position::new(std::cmp::min(self.text.len(), self.position.get() + n));
        let after = self.position;
        (before, after)
    }

    pub fn parse<T: Parse>(&mut self) -> ParseResult<T> {
        if let Some(result) = self.get_parse_result::<T>(self.position) {
            let result = result.map(|t| t.clone());
            if let Ok(t) = &result {
                self.position = t.end_position();
            }
            return result;
        }

        let start = self.position;

        self.update_expected::<T>();
        self.set_parse_result::<T>(start, Err(ParseError));
        self.level += 1;
        let result = T::parse(self);
        self.level -= 1;
        self.set_parse_result(start, result.clone());

        if result.is_err() {
            self.position = start;
        }
        result
    }

    pub fn last_expected(&self) -> Option<&Expected> {
        self.expected.as_ref()
    }

    pub fn parsed_items<T: Parse>(&self) -> impl Iterator<Item = (Position, &T)> {
        self.memo
            .get(&TypeId::of::<T>())
            .into_iter()
            .flat_map(|map| {
                map.iter().filter_map(|(position, result)| match result {
                    Ok(t) => Some((*position, t.downcast_ref::<T>().expect("unreachable"))),
                    Err(_) => None,
                })
            })
    }

    fn update_expected<T: Parse>(&mut self) {
        let expected = if let Some(mut existing) = self.expected.take() {
            if existing.position < self.position {
                Expected::new::<T>(self.position, self.level)
            } else {
                if self.position == existing.position && existing.level == self.level {
                    existing.add_item::<T>();
                }
                existing
            }
        } else {
            Expected::new::<T>(self.position, self.level)
        };
        self.expected = Some(expected);
    }

    fn set_parse_result<T: Parse>(&mut self, position: Position, result: ParseResult<T>) {
        self.memo
            .entry(TypeId::of::<T>())
            .or_default()
            .insert(position, result.map(|t| Box::new(t) as Box<dyn Any>));
    }

    fn get_parse_result<T: Parse>(&self, position: Position) -> Option<ParseResult<&T>> {
        self.memo
            .get(&TypeId::of::<T>())
            .and_then(|map| map.get(&position))
            .map(|result| match result {
                Ok(t) => Ok(t.downcast_ref::<T>().expect("unreachable")),
                Err(e) => Err(*e),
            })
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ParseError;

pub type ParseResult<T> = Result<T, ParseError>;
