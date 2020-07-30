use std::error::Error as StdError;
use std::fmt::{self, Display};

#[derive(Debug, PartialEq)]
pub struct ParseIntError;

impl StdError for ParseIntError {}

impl Display for ParseIntError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error parsing int from byte string")
    }
}

pub trait Number: Sized + Copy {
    fn from(n: u32) -> Self;
    fn checked_add(self, rhs: Self) -> Option<Self>;
    fn checked_sub(self, rhs: Self) -> Option<Self>;
    fn checked_mul(self, rhs: Self) -> Option<Self>;
    fn is_signed_type() -> bool;
}

macro_rules! impl_number_for {
    ($($t:ty),+) => {
        $(
            impl Number for $t {
                fn from(n: u32) -> Self {
                    n as Self
                }

                fn checked_add(self, rhs: Self) -> Option<Self> {
                    self.checked_add(rhs)
                }

                fn checked_sub(self, rhs: Self) -> Option<Self> {
                    self.checked_sub(rhs)
                }

                fn checked_mul(self, rhs: Self) -> Option<Self> {
                    self.checked_mul(rhs)
                }

                fn is_signed_type() -> bool {
                    let zero = 0 as $t;
                    let min_value = <$t>::min_value();
                    zero > min_value
                }
            }
        )+
    }
}

impl_number_for!(usize, u8, u16, u32, u64, isize, i8, i16, i32, i64);

// Based heavily on Rust std lib's from_str_radix
pub(crate) fn from_bytes<T: Number>(string: &[u8]) -> Result<T, ParseIntError> {
    if string.is_empty() {
        return Err(ParseIntError);
    }

    let (digits, is_positive) = match string[0] {
        b'+' => (&string[1..], true),
        b'-' if T::is_signed_type() => (&string[1..], false),
        _ => (&string[..], true),
    };

    let mut result: T = T::from(0);

    for &c in digits {
        let digit: u32 = (c as char).to_digit(10).ok_or(ParseIntError)?;
        result = result.checked_mul(T::from(10)).ok_or(ParseIntError)?;

        if is_positive {
            result = result.checked_add(T::from(digit)).ok_or(ParseIntError)?;
        } else {
            result = result.checked_sub(T::from(digit)).ok_or(ParseIntError)?;
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_bytes_signed() {
        // Success cases
        assert_eq!(from_bytes::<isize>(b"0"), Ok(0));
        assert_eq!(from_bytes::<isize>(b"1"), Ok(1));
        assert_eq!(from_bytes::<isize>(b"+0"), Ok(0));
        assert_eq!(from_bytes::<isize>(b"-0"), Ok(0));
        assert_eq!(from_bytes::<isize>(b"+1"), Ok(1));
        assert_eq!(from_bytes::<isize>(b"-1"), Ok(-1));
        assert_eq!(
            from_bytes::<isize>(b"+9223372036854775807"),
            Ok(<isize>::max_value())
        );
        assert_eq!(
            from_bytes::<isize>(b"-9223372036854775808"),
            Ok(<isize>::min_value())
        );

        // Error cases
        assert_eq!(from_bytes::<isize>(b""), Err(ParseIntError));
        assert_eq!(from_bytes::<isize>(b"x"), Err(ParseIntError));

        // The final mul by the radix will overflow
        assert_eq!(
            from_bytes::<isize>(b"92233720368547758071"),
            Err(ParseIntError)
        );

        // The adding the final 8 digit will overflow
        assert_eq!(
            from_bytes::<isize>(b"9223372036854775808"),
            Err(ParseIntError)
        );
    }

    #[test]
    fn test_from_bytes_unsigned() {
        // Success cases
        assert_eq!(from_bytes::<usize>(b"0"), Ok(0));
        assert_eq!(from_bytes::<usize>(b"1"), Ok(1));
        assert_eq!(from_bytes::<usize>(b"+0"), Ok(0));
        assert_eq!(from_bytes::<usize>(b"+1"), Ok(1));
        assert_eq!(
            from_bytes::<usize>(b"+18446744073709551615"),
            Ok(<usize>::max_value())
        );
        assert_eq!(from_bytes::<usize>(b"0"), Ok(<usize>::min_value()));

        // Error cases
        assert_eq!(from_bytes::<usize>(b"-0"), Err(ParseIntError));
        assert_eq!(from_bytes::<usize>(b"-1"), Err(ParseIntError));
        assert_eq!(from_bytes::<usize>(b""), Err(ParseIntError));
        assert_eq!(from_bytes::<usize>(b"x"), Err(ParseIntError));

        // The final mul by the radix will overflow
        assert_eq!(
            from_bytes::<usize>(b"184467440737095516151"),
            Err(ParseIntError)
        );
    }

    #[test]
    fn test_supported_types() {
        assert_eq!(from_bytes::<usize>(b"0"), Ok(0));
        assert_eq!(from_bytes::<u8>(b"0"), Ok(0));
        assert_eq!(from_bytes::<u16>(b"0"), Ok(0));
        assert_eq!(from_bytes::<u32>(b"0"), Ok(0));
        assert_eq!(from_bytes::<u64>(b"0"), Ok(0));

        assert_eq!(from_bytes::<isize>(b"0"), Ok(0));
        assert_eq!(from_bytes::<i8>(b"0"), Ok(0));
        assert_eq!(from_bytes::<i16>(b"0"), Ok(0));
        assert_eq!(from_bytes::<i32>(b"0"), Ok(0));
        assert_eq!(from_bytes::<i64>(b"0"), Ok(0));
    }

    #[test]
    fn test_parse_int_error() {
        fn expect_std_error(_: impl std::error::Error) {}
        expect_std_error(ParseIntError);
    }
}
