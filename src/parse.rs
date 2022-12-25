use crate::{Position, Span};
use std::{
    any::{Any, TypeId},
    borrow::{Borrow, Cow},
    cmp::Ordering,
    collections::{BTreeMap, HashMap},
};

pub use textparse_derive::Parse;

/// This traits allows for parsing an item from text.
pub trait Parse: 'static + Span + Clone + Sized {
    /// Parses an item.
    ///
    /// `None` means parse failure.
    fn parse(parser: &mut Parser) -> Option<Self>;

    /// Name of the item to be parsed.
    fn name() -> Option<fn() -> String> {
        None
    }
}

impl<T: Parse> Parse for Box<T> {
    fn parse(parser: &mut Parser) -> Option<Self> {
        parser.parse().map(Box::new)
    }

    fn name() -> Option<fn() -> String> {
        T::name()
    }
}

impl<T0: Parse, T1: Parse> Parse for (T0, T1) {
    fn parse(parser: &mut Parser) -> Option<Self> {
        Some((parser.parse()?, parser.parse()?))
    }
}

impl<T0: Parse, T1: Parse, T2: Parse> Parse for (T0, T1, T2) {
    fn parse(parser: &mut Parser) -> Option<Self> {
        Some((parser.parse()?, parser.parse()?, parser.parse()?))
    }
}

impl<T0: Parse, T1: Parse, T2: Parse, T3: Parse> Parse for (T0, T1, T2, T3) {
    fn parse(parser: &mut Parser) -> Option<Self> {
        Some((
            parser.parse()?,
            parser.parse()?,
            parser.parse()?,
            parser.parse()?,
        ))
    }
}

impl<T0: Parse, T1: Parse, T2: Parse, T3: Parse, T4: Parse> Parse for (T0, T1, T2, T3, T4) {
    fn parse(parser: &mut Parser) -> Option<Self> {
        Some((
            parser.parse()?,
            parser.parse()?,
            parser.parse()?,
            parser.parse()?,
            parser.parse()?,
        ))
    }
}

impl<T0: Parse, T1: Parse, T2: Parse, T3: Parse, T4: Parse, T5: Parse> Parse
    for (T0, T1, T2, T3, T4, T5)
{
    fn parse(parser: &mut Parser) -> Option<Self> {
        Some((
            parser.parse()?,
            parser.parse()?,
            parser.parse()?,
            parser.parse()?,
            parser.parse()?,
            parser.parse()?,
        ))
    }
}

#[derive(Debug)]
pub struct ErrorMessageBuilder<'a> {
    text: &'a str,
    expected: &'a Expected,
    filename: String,
}

impl<'a> ErrorMessageBuilder<'a> {
    fn new(text: &'a str, expected: &'a Expected) -> Self {
        Self {
            text,
            expected,
            filename: "<UNKNOWN>".to_owned(),
        }
    }

    pub fn filename(mut self, filename: &str) -> Self {
        self.filename = filename.to_owned();
        self
    }

    pub fn build(self) -> String {
        self.try_build().expect("unreachable")
    }

    fn try_build(self) -> Result<String, std::io::Error> {
        let offset = self.expected.position.get();
        let (line, column) = self.expected.position.line_and_column(self.text);

        let mut s = String::new();

        let mut expected_items = self.expected.items().collect::<Vec<_>>();
        expected_items.sort();
        match expected_items.len() {
            0 => {}
            1 => {
                s += &format!("expected {}", expected_items[0]);
            }
            n => {
                s += &format!("expected one of {}", expected_items[0]);
                for (i, item) in expected_items.iter().enumerate().skip(1) {
                    if i + 1 == n {
                        s += &format!(", or {item}");
                    } else {
                        s += &format!(", {item}");
                    }
                }
            }
        }
        let expected_message = s.clone();

        if offset == self.text.len() {
            s += ", reached EOS";
        }
        s += "\n";

        s += &format!("  --> {}:{line}:{column}\n", self.filename);

        let line_len = format!("{line}").len();
        s += &format!("{:line_len$} |\n", ' ');
        s += &format!(
            "{line} | {}\n",
            self.text[offset + 1 - column..]
                .lines()
                .next()
                .unwrap_or("")
        );
        s += &format!("{:line_len$} | {:>column$} {expected_message}\n", ' ', '^');
        Ok(s)
    }
}

#[derive(Debug, Default)]
struct Expected {
    position: Position,
    level: usize,
    expected_items: HashMap<TypeId, fn() -> String>,
}

impl Expected {
    fn new<T: Parse>(position: Position, level: usize, name: fn() -> String) -> Self {
        let mut this = Self {
            position,
            level,
            expected_items: Default::default(),
        };
        this.add_item::<T>(name);
        this
    }

    fn add_item<T: Parse>(&mut self, name: fn() -> String) {
        self.expected_items.insert(TypeId::of::<T>(), name);
    }

    pub fn items(&self) -> impl '_ + Iterator<Item = String> {
        self.expected_items.values().map(|f| f())
    }
}

/// Parser.
#[derive(Debug)]
pub struct Parser<'a> {
    text: Cow<'a, str>,
    position: Position,
    level: usize,
    expected: Expected,
    memo: HashMap<TypeId, BTreeMap<Position, Option<Box<dyn Any>>>>,
}

impl<'a> Parser<'a> {
    /// Makes a new [`Parser`] instance.
    pub fn new(text: &'a str) -> Self {
        Self {
            text: Cow::Borrowed(text),
            position: Position::default(),
            level: 0,
            expected: Expected::default(),
            memo: HashMap::default(),
        }
    }

    /// Returns the current position.
    pub fn current_position(&self) -> Position {
        self.position
    }

    /// Returns `true` if the parser has reached EOS, otherwise `false`.
    pub fn is_eos(&self) -> bool {
        self.text.len() == self.position.get()
    }

    /// Returns the full text.
    pub fn text(&self) -> &str {
        self.text.borrow()
    }

    /// Returns the remaining, un-parsed text.
    pub fn remaining_text(&self) -> &str {
        &self.text[self.position.get()..]
    }

    /// Peeks the next character.
    pub fn peek_char(&self) -> Option<char> {
        self.remaining_text().chars().next()
    }

    /// Reads the next character.
    pub fn read_char(&mut self) -> Option<char> {
        if let Some(c) = self.peek_char() {
            self.position = Position::new(self.position.get() + c.len_utf8());
            Some(c)
        } else {
            None
        }
    }

    /// Parses an item.
    pub fn parse<T: Parse>(&mut self) -> Option<T> {
        if let Some(result) = self.get_parse_result::<T>(self.position) {
            let result = result.cloned();
            if let Some(t) = &result {
                self.position = t.end_position();
            }
            return result;
        }

        let start = self.position;

        let has_name = if let Some(name) = T::name() {
            self.update_expected::<T>(name);
            true
        } else {
            false
        };
        self.set_parse_result_if_absent::<T>(start, None);
        if has_name {
            self.level += 1;
        }
        let result = T::parse(self);
        if has_name {
            self.level -= 1;
        }

        self.set_parse_result(start, result.clone());

        if result.is_none() {
            self.position = start;
        }
        result
    }

    /// Returns parsed items of which type is `T`.
    pub fn parsed_items<T: Parse>(&self) -> impl Iterator<Item = (Position, &T)> {
        self.memo
            .get(&TypeId::of::<T>())
            .into_iter()
            .flat_map(|map| {
                map.iter().filter_map(|(position, result)| {
                    result
                        .as_ref()
                        .map(|item| (*position, item.downcast_ref::<T>().expect("unreachable")))
                })
            })
    }

    // TODO
    pub fn error_message_builder(&self) -> ErrorMessageBuilder {
        ErrorMessageBuilder::new(self.text(), &self.expected)
    }

    // TODO: private(?)
    pub fn into_owned(self) -> Parser<'static> {
        Parser {
            text: Cow::Owned(self.text.into_owned()),
            position: self.position,
            level: self.level,
            expected: self.expected,
            memo: self.memo,
        }
    }

    fn update_expected<T: Parse>(&mut self, name: fn() -> String) {
        match (
            self.expected.position.cmp(&self.position),
            self.expected.level.cmp(&self.level),
        ) {
            (Ordering::Equal, Ordering::Equal) => {
                self.expected.add_item::<T>(name);
            }
            (Ordering::Less, _) | (Ordering::Equal, Ordering::Greater) => {
                self.expected = Expected::new::<T>(self.position, self.level, name);
            }
            _ => {}
        }
    }

    fn set_parse_result<T: Parse>(&mut self, position: Position, result: Option<T>) {
        self.memo
            .entry(TypeId::of::<T>())
            .or_default()
            .insert(position, result.map(|t| Box::new(t) as Box<dyn Any>));
    }

    fn set_parse_result_if_absent<T: Parse>(&mut self, position: Position, result: Option<T>) {
        self.memo
            .entry(TypeId::of::<T>())
            .or_default()
            .entry(position)
            .or_insert_with(|| result.map(|t| Box::new(t) as Box<dyn Any>));
    }

    fn get_parse_result<T: Parse>(&self, position: Position) -> Option<Option<&T>> {
        self.memo
            .get(&TypeId::of::<T>())
            .and_then(|map| map.get(&position))
            .map(|result| {
                result
                    .as_ref()
                    .map(|item| item.downcast_ref::<T>().expect("unreachable"))
            })
    }
}
