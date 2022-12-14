use std::io::Read;
use textparse::{
    components::{AnyChar, Char, Digit, Eos, Items, NonEmpty, Not, Str, While, Whitespace},
    Parse, Parser, Position, Span,
};

fn main() -> anyhow::Result<()> {
    let mut text = String::new();
    std::io::stdin().read_to_string(&mut text)?;

    let mut parser = Parser::new(&text);
    if parser.parse::<(JsonValue, Eos)>().is_some() {
        println!("OK: the input string is a JSON text.");
    } else {
        println!("Error: {}", parser.into_parse_error().file_path("<STDIN>"));
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
struct JsonNull(Str<'n', 'u', 'l', 'l'>);

#[derive(Clone, Span, Parse)]
#[parse(name = "a JSON string")]
struct JsonString(Char<'"'>, While<(Not<Char<'"'>>, AnyChar)>, Char<'"'>);

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
struct WithoutWhitespaces<T>(While<Whitespace>, T, While<Whitespace>);
