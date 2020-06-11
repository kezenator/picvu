use crate::ParseError;

pub fn encode(val: i64, suffix: &'static str) -> String
{
    let mut plain = val.to_string();
    plain.push_str(suffix);
    
    // Base64 encoding turns groups of 3 bytes
    // into groups of 4 characters - we don't want the
    // tell-tale equals suffix on the encoded string, so
    // pad the input string to a multiple of 3 bytes

    while (plain.len() % 3) != 0
    {
        plain.push('=');
    }

    data_encoding::BASE64URL.encode(plain.as_bytes())
}

pub fn decode(encoded: &str, suffix: &'static str) -> Result<i64, ParseError>
{
    if let Ok(bytes) = data_encoding::BASE64URL.decode(encoded.as_bytes())
    {
        if let Ok(plain) = String::from_utf8(bytes)
        {
            let val_str = plain.trim_end_matches('=').trim_end_matches(suffix);

            if let Ok(val) = val_str.parse()
            {
                let canonical = encode(val, suffix);

                if canonical == encoded
                {
                    return Ok(val);
                }
            }
        }
    }
    
    Err(ParseError::new(format!("Invalid ID string: {}", encoded)))
}

#[cfg(test)]
mod tests
{
    use super::*;

    pub fn test_round_trip(num: i64, suffix: &'static str, enc: &'static str)
    {
        assert_eq!(encode(num, suffix), enc.to_owned());
        assert_eq!(decode(enc, suffix), Ok(num));
    }

    #[test]
    pub fn test_ids()
    {
        test_round_trip(i64::MAX, "o", "OTIyMzM3MjAzNjg1NDc3NTgwN289");
        test_round_trip(0, "o", "MG89");
        test_round_trip(1, "o", "MW89");
        test_round_trip(2, "o", "Mm89");
        test_round_trip(100, "o", "MTAwbz09");
        test_round_trip(1000, "o", "MTAwMG89");
        test_round_trip(1000000, "o", "MTAwMDAwMG89");
        test_round_trip(100000000, "o", "MTAwMDAwMDAwbz09");
    }
}
