//! GraphQL support for [`rust_decimal`] crate types.
//!
//! # Supported types
//!
//! | Rust type   | GraphQL scalar |
//! |-------------|----------------|
//! | [`Decimal`] | `Decimal`      |
//!
//! [`Decimal`]: rust_decimal::Decimal

use crate::{ScalarValue, graphql_scalar};

/// 128 bit representation of a fixed-precision decimal number.
///
/// The finite set of values of `Decimal` scalar are of the form
/// m / 10<sup>e</sup>, where m is an integer such that
/// -2<sup>96</sup> < m < 2<sup>96</sup>, and e is an integer between 0 and 28
/// inclusive.
///
/// Always serializes as `String`. But may be deserialized from `Int` and
/// `Float` values too. It's not recommended to deserialize from a `Float`
/// directly, as the floating point representation may be unexpected.
///
/// See also [`rust_decimal`] crate for details.
///
/// [`rust_decimal`]: https://docs.rs/rust_decimal
#[graphql_scalar]
#[graphql(
    with = rust_decimal_scalar,
    to_output_with = ScalarValue::from_displayable,
    parse_token(i32, f64, String),
    specified_by_url = "https://docs.rs/rust_decimal",
)]
type Decimal = rust_decimal::Decimal;

mod rust_decimal_scalar {
    use super::Decimal;
    use crate::{Scalar, ScalarValue};

    pub(super) fn from_input(v: &Scalar<impl ScalarValue>) -> Result<Decimal, Box<str>> {
        if let Some(i) = v.try_to_int() {
            Ok(Decimal::from(i))
        } else if let Some(f) = v.try_to_float() {
            Decimal::try_from(f)
                .map_err(|e| format!("Failed to parse `Decimal` from `Float`: {e}").into())
        } else {
            v.try_to::<&str>()
                .map_err(|e| e.to_string().into())
                .and_then(|s| {
                    s.parse::<Decimal>()
                        .map_err(|e| format!("Failed to parse `Decimal` from `String`: {e}").into())
                })
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{FromInputValue as _, InputValue, ToInputValue as _, graphql_input_value};

    use super::Decimal;

    #[test]
    fn parses_correct_input() {
        for (input, expected) in [
            (graphql_input_value!("4.20"), "4.20"),
            (graphql_input_value!("0"), "0"),
            (graphql_input_value!("999.999999999"), "999.999999999"),
            (graphql_input_value!("875533788"), "875533788"),
            (graphql_input_value!(123), "123"),
            (graphql_input_value!(0), "0"),
            (graphql_input_value!(43.44), "43.44"),
        ] {
            let input: InputValue = input;
            let parsed = Decimal::from_input_value(&input);
            let expected = expected.parse::<Decimal>().unwrap();

            assert!(
                parsed.is_ok(),
                "failed to parse `{input:?}`: {:?}",
                parsed.unwrap_err(),
            );
            assert_eq!(parsed.unwrap(), expected, "input: {input:?}");
        }
    }

    #[test]
    fn fails_on_invalid_input() {
        for input in [
            graphql_input_value!(""),
            graphql_input_value!("0,0"),
            graphql_input_value!("12,"),
            graphql_input_value!("1996-12-19T14:23:43"),
            graphql_input_value!("99999999999999999999999999999999999999"),
            graphql_input_value!("99999999999999999999999999999999999999.99"),
            graphql_input_value!("i'm not even a number"),
            graphql_input_value!(null),
            graphql_input_value!(false),
        ] {
            let input: InputValue = input;
            let parsed = Decimal::from_input_value(&input);

            assert!(parsed.is_err(), "allows input: {input:?}");
        }
    }

    #[test]
    fn formats_correctly() {
        for raw in ["4.20", "0", "999.999999999", "875533788", "123", "43.44"] {
            let actual: InputValue = raw.parse::<Decimal>().unwrap().to_input_value();

            assert_eq!(actual, graphql_input_value!((raw)), "on value: {raw}");
        }
    }
}
