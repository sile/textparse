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

#[derive(Clone, Span, Parse)]
struct JsonValue(WithoutWhitespaces<JsonValueInner>);

#[derive(Clone, Span, Parse)]
#[parse(name = "a JSON value")]
enum JsonValueInner {
    Null(JsonNull),
    String(JsonString),
    Number(JsonNumber),
    Array(JsonArray),
    Object(JsonObject),
}

#[derive(Clone, Span, Parse)]
#[parse(name = "`null`")]
struct JsonNull(StartsWith<Null>);

#[derive(Clone, Span, Parse)]
#[parse(name = "a JSON string")]
struct JsonString(Char<'"'>, While<IsStringContent>, Char<'"'>);

#[derive(Clone, Span, Parse)]
#[parse(name = "a JSON number")]
struct JsonNumber(NonEmpty<While<IsDigit>>);

#[derive(Clone, Span, Parse)]
#[parse(name = "a JSON array")]
struct JsonArray(Char<'['>, Csv<JsonValue>, Char<']'>);

#[derive(Clone, Span, Parse)]
#[parse(name = "a JSON object")]
struct JsonObject(Char<'{'>, Csv<JsonObjectItem>, Char<'}'>);

#[derive(Clone, Span, Parse)]
struct JsonObjectItem(WithoutWhitespaces<JsonString>, Char<':'>, JsonValue);

#[derive(Clone, Span, Parse)]
struct Csv<T>(Items<T, Char<','>>);

#[derive(Clone, Span, Parse)]
struct WithoutWhitespaces<T>(SkipWhitespaces, T, SkipWhitespaces);

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
