use std::collections::HashMap;

use anyhow::Result;
use winnow::{
    ascii::{digit1, float},
    combinator::{alt, delimited, opt, separated, separated_pair},
    token::take_until,
    Parser,
};

#[allow(unused)]
#[derive(Debug, Clone)]
enum JsonValue {
    Null,
    Bool(bool),
    Int(i64),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

#[allow(unused)]
#[derive(Debug)]
struct User {
    name: String,
    age: u32,
    is_student: bool,
    marks: Vec<u32>,
    address: Address,
}

#[allow(unused)]
#[derive(Debug)]
struct Address {
    city: String,
    zip: u32,
}

fn main() -> Result<()> {
    let s = r#"
    {
      "name": "John Doe",
      "age": 30,
      "is_student": false,
      "marks": [90, -80, 85],
      address: {
        "city": "London",
        "zip": 10001
      }
    }"#;

    parse_json(s).unwrap();
    // println!("{:#?}", v);

    Ok(())
}

fn parse_json(input: &str) -> winnow::Result<()> {
    let mut input = input;
    let user = parse_object.parse_next(&mut input)?;
    println!("{:?}", user);
    Ok(())
}

fn parse_null(input: &mut &str) -> winnow::Result<()> {
    "null".value(()).parse_next(input)
}

fn parse_bool(input: &mut &str) -> winnow::Result<bool> {
    alt(("true", "false")).parse_to().parse_next(input)
}

fn parse_int(input: &mut &str) -> winnow::Result<i64> {
    let sign = opt('-').map(|s| s.is_some()).parse_next(input)?;
    let num = digit1.parse_to::<i64>().parse_next(input)?;
    Ok(if sign { -num } else { num })
}

fn parse_number(input: &mut &str) -> winnow::Result<f64> {
    float.parse_next(input)
}

fn parse_string(input: &mut &str) -> winnow::Result<String> {
    let ret = delimited('"', take_until(0.., '"'), '"').parse_next(input)?;
    Ok(ret.to_string())
}

fn parse_array(input: &mut &str) -> winnow::Result<Vec<JsonValue>> {
    let parse_value = separated(0.., parse_value, ',');
    let ret = delimited('[', parse_value, ']').parse_next(input)?;
    Ok(ret)
}

fn parse_object(input: &mut &str) -> winnow::Result<HashMap<String, JsonValue>> {
    let parser_kv_pair = separated_pair(parse_string, ":", parse_value);
    let parse_kv = separated(1.., parser_kv_pair, ',');
    let ret = delimited('{', parse_kv, '}').parse_next(input)?;
    Ok(ret)
}

fn parse_value(input: &mut &str) -> winnow::Result<JsonValue> {
    alt((
        parse_null.value(JsonValue::Null),
        parse_bool.map(JsonValue::Bool),
        parse_int.map(JsonValue::Int),
        parse_number.map(JsonValue::Number),
        parse_string.map(JsonValue::String),
        parse_array.map(JsonValue::Array),
        parse_object.map(JsonValue::Object),
    ))
    .parse_next(input)
}
