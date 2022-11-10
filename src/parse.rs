use crate::{Position, Span};
use std::{
    any::{Any, TypeId},
    cmp::Ordering,
    collections::{BTreeMap, HashMap},
};

pub use textparse_derive::Parse;

pub trait Parse: 'static + Span + Clone + Sized {
    fn parse(parser: &mut Parser) -> ParseResult<Self>;

    fn name() -> Option<fn() -> String> {
        None
    }
}

impl<T: Parse> Parse for Box<T> {
    fn parse(parser: &mut Parser) -> ParseResult<Self> {
        T::parse(parser).map(Box::new)
    }

    fn name() -> Option<fn() -> String> {
        T::name()
    }
}

impl<T0: Parse, T1: Parse> Parse for (T0, T1) {
    fn parse(parser: &mut Parser) -> ParseResult<Self> {
        Ok((parser.parse()?, parser.parse()?))
    }
}

impl<T0: Parse, T1: Parse, T2: Parse> Parse for (T0, T1, T2) {
    fn parse(parser: &mut Parser) -> ParseResult<Self> {
        Ok((parser.parse()?, parser.parse()?, parser.parse()?))
    }
}

impl<T0: Parse, T1: Parse, T2: Parse, T3: Parse> Parse for (T0, T1, T2, T3) {
    fn parse(parser: &mut Parser) -> ParseResult<Self> {
        Ok((
            parser.parse()?,
            parser.parse()?,
            parser.parse()?,
            parser.parse()?,
        ))
    }
}

impl<T0: Parse, T1: Parse, T2: Parse, T3: Parse, T4: Parse> Parse for (T0, T1, T2, T3, T4) {
    fn parse(parser: &mut Parser) -> ParseResult<Self> {
        Ok((
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

    fn line_and_column(&self) -> (usize, usize) {
        let offset = self.expected.position.get();
        let mut line = 1;
        let mut column = 1;
        for c in self.text[..offset].chars() {
            if c == '\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
        }
        (line, column)
    }

    // fn expected_message(&self) -> String {
    //     let mut expected_items = self.expected.items().collect::<Vec<_>>();
    //     expected_items.sort();
    //     match expected_items.len() {
    //         0 => String::new(),
    //         1 => {
    //             s += &format!("expected {}", expected_items[0]);
    //         }
    //         n => {
    //             s += &format!("expected one of {}", expected_items[0]);
    //             for (i, item) in expected_items.iter().enumerate().skip(1) {
    //                 if i + 1 == n {
    //                     s += &format!(", or {}", item);
    //                 } else {
    //                     s += &format!(", {}", item);
    //                 }
    //             }
    //         }
    //     }
    // }

    fn try_build(self) -> Result<String, std::io::Error> {
        let offset = self.expected.position.get();
        let (line, column) = self.line_and_column();

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
                        s += &format!(", or {}", item);
                    } else {
                        s += &format!(", {}", item);
                    }
                }
            }
        }
        let expected_message = s.clone();

        if offset == self.text.len() {
            s += ", reached EOS";
        }
        s += "\n";

        s += &format!("  --> {}:{}:{}\n", self.filename, line, column);

        let line_len = format!("{}", line).len();
        s += &format!("{:line_len$} |\n", ' ');
        s += &format!(
            "{} | {}\n",
            line_len,
            self.text[offset + 1 - column..]
                .lines()
                .next()
                .expect("unreachable")
        );
        s += &format!(
            "{:line_len$} | {:>column$} {}\n",
            ' ', '^', expected_message
        );
        Ok(s)
    }
}

#[derive(Debug, Default)]
pub struct Expected {
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

    pub fn position(&self) -> Position {
        self.position
    }

    pub fn items(&self) -> impl '_ + Iterator<Item = String> {
        self.expected_items.values().map(|f| f())
    }
}

#[derive(Debug)]
pub struct Parser<'a> {
    text: &'a str,
    position: Position,
    level: usize,
    expected: Expected,
    memo: HashMap<TypeId, BTreeMap<Position, ParseResult<Box<dyn Any>>>>,
}

impl<'a> Parser<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            position: Position::default(),
            level: 0,
            expected: Expected::default(),
            memo: HashMap::default(),
        }
    }

    pub fn error_message_builder(&self) -> ErrorMessageBuilder {
        ErrorMessageBuilder::new(self.text, &self.expected)
    }

    pub fn current_position(&self) -> Position {
        self.position
    }

    pub fn set_current_position(&mut self, position: Position) {
        self.position = Position::new(std::cmp::min(self.text.len(), position.get()));
    }

    pub fn is_eos(&self) -> bool {
        self.text.len() == self.position.get()
    }

    pub fn remaining_text(&self) -> &str {
        &self.text[self.position.get()..]
    }

    pub fn peek_char(&self) -> Option<char> {
        self.remaining_text().chars().next()
    }

    pub fn read_char(&mut self) -> Option<char> {
        if let Some(c) = self.peek_char() {
            self.consume_bytes(c.len_utf8());
            Some(c)
        } else {
            None
        }
    }

    pub fn consume_chars(&mut self, n: usize) -> (Position, Position) {
        let n = self
            .remaining_text()
            .chars()
            .take(n)
            .map(|c| c.len_utf8())
            .sum();
        self.consume_bytes(n)
    }

    pub fn consume_bytes(&mut self, n: usize) -> (Position, Position) {
        let before = self.position;
        self.position = Position::new(std::cmp::min(self.text.len(), self.position.get() + n));
        let after = self.position;
        (before, after)
    }

    pub fn peek<T: Parse>(&mut self) -> ParseResult<T> {
        let position = self.position;
        let result = self.parse();
        self.position = position;
        result
    }

    pub fn peek_without_memo<T: Parse>(&mut self) -> ParseResult<T> {
        let position = self.position;
        let result = self.parse_without_memo();
        self.position = position;
        result
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

        let has_name = if let Some(name) = T::name() {
            self.update_expected::<T>(name);
            true
        } else {
            false
        };
        self.set_parse_result_if_absent::<T>(start, Err(ParseError));
        if has_name {
            self.level += 1;
        }
        let result = T::parse(self);
        if has_name {
            self.level -= 1;
        }
        self.set_parse_result(start, result.clone());

        if result.is_err() {
            self.position = start;
        }
        result
    }

    // TODO: rename
    pub fn parse_without_memo<T: Parse>(&mut self) -> ParseResult<T> {
        let start = self.position;

        let has_name = if let Some(name) = T::name() {
            self.update_expected::<T>(name);
            true
        } else {
            false
        };
        self.set_parse_result_if_absent::<T>(start, Err(ParseError));
        if has_name {
            self.level += 1;
        }
        let result = T::parse(self);
        if has_name {
            self.level -= 1;
        }
        self.set_parse_result(start, result.clone());

        if result.is_err() {
            self.position = start;
        }
        result
    }

    pub fn expected(&self) -> &Expected {
        &self.expected
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

    fn update_expected<T: Parse>(&mut self, name: fn() -> String) {
        match (
            self.expected.position.cmp(&self.position),
            self.expected.level.cmp(&self.level),
        ) {
            (Ordering::Equal, Ordering::Equal) => {
                self.expected.add_item::<T>(name);
            }
            (Ordering::Less, _) | (Ordering::Equal, Ordering::Less) => {
                self.expected = Expected::new::<T>(self.position, self.level, name);
            }
            _ => {}
        }
    }

    pub fn set_parsed_item<T: Parse>(&mut self, item: T) {
        self.set_parse_result(item.start_position(), Ok(item));
    }

    pub fn set_parse_result<T: Parse>(&mut self, position: Position, result: ParseResult<T>) {
        self.memo
            .entry(TypeId::of::<T>())
            .or_default()
            .insert(position, result.map(|t| Box::new(t) as Box<dyn Any>));
    }

    fn set_parse_result_if_absent<T: Parse>(&mut self, position: Position, result: ParseResult<T>) {
        self.memo
            .entry(TypeId::of::<T>())
            .or_default()
            .entry(position)
            .or_insert_with(|| result.map(|t| Box::new(t) as Box<dyn Any>));
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
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ParseError;

pub type ParseResult<T> = Result<T, ParseError>;
