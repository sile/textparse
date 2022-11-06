use orfail::{OrFail, Result};
use std::io::Read;
use textparse::{
    components::{Char, Predicate, While},
    Parse, ParseResult, Parser, Position, Span,
};

fn main() -> Result<()> {
    let mut text = String::new();
    std::io::stdin().read_to_string(&mut text).or_fail()?;

    let mut parser = Parser::new(&text);
    if parser.parse::<JsonString>().is_ok() {
        println!("[OK] Input is a JSON text.");
    } else {
        // TODO
        println!("[NG] Input is not a JSON text.");
    }
    Ok(())
}

#[derive(Debug, Clone, Span, Parse)]
struct JsonString {
    start: Char<'"'>,
    content: While<IsStringContent>,
    end: Char<'"'>,
}

#[derive(Debug)]
pub struct IsStringContent;

impl Predicate for IsStringContent {
    fn is(c: char) -> bool {
        c != '"'
    }
}
