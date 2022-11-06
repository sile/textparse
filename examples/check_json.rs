use orfail::{OrFail, Result};
use std::io::Read;
use textparse::{
    components::{Char, Items, NonEmpty, Predicate, SkipWhitespaces, StartsWith, StaticStr, While},
    Parse, ParseResult, Parser, Position, Span,
};

fn main() -> Result<()> {
    let mut text = String::new();
    std::io::stdin().read_to_string(&mut text).or_fail()?;

    let mut parser = Parser::new(&text);
    if parser.parse::<JsonValue>().is_ok() && parser.is_eos() {
        println!("[OK] Input is a JSON text.");
    } else {
        // TODO
        println!("[NG] Input is not a JSON text.");
        let expected = parser.expected();
        println!("Position: {:?}", expected.position());
        println!("Items: {:?}", expected.items().collect::<Vec<_>>());
    }
    Ok(())
}

// TODO: use tuple struct
#[derive(Clone, Span, Parse)]
struct JsonValue {
    _skip0: SkipWhitespaces,
    value: JsonValueInner,
    _skip1: SkipWhitespaces,
}

#[derive(Clone, Span, Parse)]
enum JsonValueInner {
    Null(JsonNull),
    String(JsonString),
    Number(JsonNumber),
    Array(JsonArray),
    Object(JsonObject),
}

#[derive(Clone, Span, Parse)]
struct JsonObject {
    open: Char<'{'>,
    items: Items<JsonObjectItem, Char<','>>,
    close: Char<'}'>,
}

#[derive(Clone, Span, Parse)]
struct JsonObjectItem {
    _skip0: SkipWhitespaces,
    key: JsonString,
    _skip1: SkipWhitespaces,
    delimiter: Char<':'>,
    value: JsonValue,
}

#[derive(Clone, Span, Parse)]
struct JsonArray {
    open: Char<'['>,
    items: Items<JsonValue, Char<','>>,
    close: Char<']'>,
}

#[derive(Clone, Span, Parse)]
struct JsonNull {
    null: StartsWith<Null>,
}

#[derive(Clone, Span, Parse)]
struct JsonString {
    start: Char<'"'>,
    content: While<IsStringContent>,
    end: Char<'"'>,
}

#[derive(Clone, Span, Parse)]
struct JsonNumber {
    digits: NonEmpty<While<IsDigit>>,
}

struct Null;
impl StaticStr for Null {
    fn static_str() -> &'static str {
        "null"
    }
}

struct IsStringContent;
impl Predicate for IsStringContent {
    fn is(c: char) -> bool {
        c != '"'
    }
}

struct IsDigit;
impl Predicate for IsDigit {
    fn is(c: char) -> bool {
        c.is_ascii_digit()
    }
}
