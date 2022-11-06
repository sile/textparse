use orfail::{OrFail, Result};
use std::io::Read;
use textparse::{
    components::{
        Char, Eos, Items, NonEmpty, SkipWhitespaces, StartsWith, StaticStr, Until, While,
    },
    Parse, ParseError, ParseResult, Parser, Position, Span,
};

fn main() -> Result<()> {
    let mut text = String::new();
    std::io::stdin().read_to_string(&mut text).or_fail()?;

    let mut parser = Parser::new(&text);
    if parser.parse::<(JsonValue, SkipWhitespaces, Eos)>().is_ok() {
        println!("OK: the input string is a JSON text.");
    } else {
        println!(
            "Error: {}",
            parser.error_message_builder().filename("<STDIN>").build()
        );
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
struct JsonString(Char<'"'>, Until<Char<'"'>>, Char<'"'>);

#[derive(Clone, Span, Parse)]
#[parse(name = "a JSON number")]
struct JsonNumber(NonEmpty<While<Digit>>);

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

#[derive(Debug, Clone, Span)]
struct Digit(Position, Position);
impl Parse for Digit {
    fn parse(parser: &mut Parser) -> ParseResult<Self> {
        parser
            .peek_char()
            .filter(|c| c.is_ascii_digit())
            .ok_or(ParseError)?;
        let (start, end) = parser.consume_chars(1);
        Ok(Self(start, end))
    }
}
