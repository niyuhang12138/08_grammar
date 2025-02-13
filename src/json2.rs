use anyhow::anyhow;
use pest::{iterators::Pair, Parser};
use pest_derive::Parser;
use std::collections::HashMap;

#[derive(Parser)]
#[grammar = "json.pest"]
struct JsonParser;

#[allow(unused)]
#[derive(Debug, PartialEq)]
enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

fn main() -> anyhow::Result<()> {
    let s = r#"{
      "name": "John Doe",
      "age": 30,
      "is_student": false,
      "marks": [90.0, -80.0, 85.1],
      "address": {
          "city": "New York",
          "zip": 10001
      }
    }"#;

    let parsed = JsonParser::parse(Rule::json, s)?.next().unwrap();
    let parse_value = parse_value(parsed)?;
    println!("{:#?}", parse_value);

    Ok(())
}

fn parse_value(pair: Pair<Rule>) -> anyhow::Result<JsonValue> {
    let v = match pair.as_rule() {
        Rule::null => JsonValue::Null,
        Rule::bool => JsonValue::Bool(pair.as_str().parse()?),
        Rule::number => JsonValue::Number(pair.as_str().parse()?),
        Rule::chars => JsonValue::String(pair.as_str().to_string()),
        Rule::array => {
            let mut values = vec![];
            for inner_pair in pair.into_inner() {
                values.push(parse_value(inner_pair)?);
            }
            JsonValue::Array(values)
        }
        Rule::object => {
            let inner = pair.into_inner();
            let values = inner.map(|pair| {
                let mut inner = pair.into_inner();
                let key = inner
                    .next()
                    .map(|p| p.as_str().to_string())
                    .ok_or_else(|| anyhow!("key not found"))?;
                let value = parse_value(inner.next().ok_or_else(|| anyhow!("value not found"))?)?;
                Ok((key, value))
            });
            JsonValue::Object(values.collect::<anyhow::Result<HashMap<_, _>>>()?)
        }
        Rule::value => {
            let inner = pair
                .into_inner()
                .next()
                .ok_or_else(|| anyhow!("value not found"))?;
            parse_value(inner)?
        }
        _ => unreachable!(),
    };
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn pest_parse_null_should_work() -> Result<()> {
        let input = "null";
        let parsed = JsonParser::parse(Rule::null, input)?.next().unwrap();
        let result = parse_value(parsed)?;
        assert_eq!(JsonValue::Null, result);

        Ok(())
    }

    #[test]
    fn pest_parse_bool_should_work() -> Result<()> {
        let input = "true";
        let parsed = JsonParser::parse(Rule::bool, input)?.next().unwrap();
        let result = parse_value(parsed)?;
        assert_eq!(JsonValue::Bool(true), result);

        Ok(())
    }

    #[test]
    fn pest_parse_number_should_work() -> Result<()> {
        let input = "123";
        let parsed = JsonParser::parse(Rule::number, input)?.next().unwrap();
        let v = parse_value(parsed)?;
        assert_eq!(JsonValue::Number(123.0), v);

        let input = "-123";
        let parsed = JsonParser::parse(Rule::number, input)?.next().unwrap();
        let v = parse_value(parsed)?;
        assert_eq!(JsonValue::Number(-123.0), v);

        let input = "123.11";
        let parsed = JsonParser::parse(Rule::number, input)?.next().unwrap();
        let v = parse_value(parsed)?;
        assert_eq!(JsonValue::Number(123.11), v);

        let input = "-123.11";
        let parsed = JsonParser::parse(Rule::number, input)?.next().unwrap();
        let v = parse_value(parsed)?;
        assert_eq!(JsonValue::Number(-123.11), v);

        Ok(())
    }

    #[test]
    fn pest_parse_string_should_work() -> Result<()> {
        let input = "hello";
        let parsed = JsonParser::parse(Rule::chars, input)?.next().unwrap();
        let v = parse_value(parsed)?;
        assert_eq!(JsonValue::String("hello".to_string()), v);

        Ok(())
    }

    #[test]
    fn pest_parse_array_should_work() -> Result<()> {
        let input = r#"[1, 2, 3]"#;
        let parsed = JsonParser::parse(Rule::array, input)?.next().unwrap();
        let result = parse_value(parsed)?;
        assert_eq!(
            JsonValue::Array(vec![
                JsonValue::Number(1.0),
                JsonValue::Number(2.0),
                JsonValue::Number(3.0)
            ]),
            result
        );

        Ok(())
    }

    #[test]
    fn pest_parse_object_should_work() -> Result<()> {
        let input = r#"{"a": 1, "b": 2, "c": 3}"#;
        let parsed = JsonParser::parse(Rule::object, input)?.next().unwrap();
        println!("{parsed:#?}");
        let result = parse_value(parsed)?;
        assert_eq!(
            JsonValue::Object(
                vec![
                    ("a".to_string(), JsonValue::Number(1.0)),
                    ("b".to_string(), JsonValue::Number(2.0)),
                    ("c".to_string(), JsonValue::Number(3.0))
                ]
                .into_iter()
                .collect()
            ),
            result
        );

        Ok(())
    }
}
