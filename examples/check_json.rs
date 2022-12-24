use std::io::Read;
use textparse::{
    components::{
        AnyChar, Char, Digit, Eos, Items, NonEmpty, Not, StaticStr, Str, While, Whitespace,
    },
    Parse, ParseResult, Parser, Position, Span,
};

fn main() -> anyhow::Result<()> {
    let mut text = String::new();
    std::io::stdin().read_to_string(&mut text)?;

    let mut parser = Parser::new(&text);
    if parser.parse::<(JsonValue, Eos)>().is_ok() {
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
struct JsonNull(Str<Null>);

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

struct Null;
impl StaticStr for Null {
    fn static_str() -> &'static str {
        "null"
    }
}
