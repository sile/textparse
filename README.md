textparse
=========

[![textparse](https://img.shields.io/crates/v/textparse.svg)](https://crates.io/crates/textparse)
[![Documentation](https://docs.rs/textparse/badge.svg)](https://docs.rs/textparse)
[![Actions Status](https://github.com/sile/textparse/workflows/CI/badge.svg)](https://github.com/sile/textparse/actions)
![License](https://img.shields.io/crates/l/textparse)

A Rust library to declaratively implement parsers that are based on Packrat Parsing.

Examples
--------

The following code implements a parser for a JSON subset format:
```rust
use textparse::{
    components::{AnyChar, Char, Digit, Eos, Items, NonEmpty, Not, StaticStr, Str, While, Whitespace},
    Parse, ParseResult, Parser, Position, Span,
};

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
```

You can run the above parser via [examples/check_json.rs](examples/check_json.rs) as follows:
```console
$ echo '["foo",null,{"key": "value"}, 123]' | cargo run --example check_json
OK: the input string is a JSON text.

$ echo '["foo" null]' | cargo run --example check_json
Error: expected one of ',', or ']'
  --> <STDIN>:1:8
  |
1 | ["foo" null]
  |        ^ expected one of ',', or ']'
```
