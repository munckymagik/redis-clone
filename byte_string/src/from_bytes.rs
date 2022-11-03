use std::error::Error as StdError;
use std::fmt::{self, Display};
use std::ops::Neg;

#[derive(Debug, PartialEq, Eq)]
pub struct ParseIntError;

impl StdError for ParseIntError {}

impl Display for ParseIntError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error parsing int from byte string")
    }
}

pub trait Number: Sized + Neg<Output = Self> + Copy {
    fn from(n: u32) -> Self;
    fn checked_add(self, rhs: Self) -> Option<Self>;
    fn checked_mul(self, rhs: Self) -> Option<Self>;
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

                fn checked_mul(self, rhs: Self) -> Option<Self> {
                    self.checked_mul(rhs)
                }
            }
        )+
    }
}

impl_number_for!(i64, isize);

// Based heavily on Rust std lib's from_str_radix
pub(crate) fn from_bytes<T: Number>(string: &[u8]) -> Result<T, ParseIntError> {
    if string.is_empty() {
        return Err(ParseIntError);
    }

    let (digits, sign_factor): (_, T) = match string[0] {
        b'+' => (&string[1..], T::from(1)),
        b'-' => (&string[1..], T::from(1).neg()),
        _ => (string, T::from(1)),
    };

    let mut result: T = T::from(0);

    for &c in digits {
        let digit: u32 = (c as char).to_digit(10).ok_or(ParseIntError)?;
        let signed_digit = T::from(digit)
            .checked_mul(sign_factor)
            .ok_or(ParseIntError)?;

        result = result.checked_mul(T::from(10)).ok_or(ParseIntError)?;
        result = result.checked_add(signed_digit).ok_or(ParseIntError)?;
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_for_type {
        ($t:ty, $i:ident) => {
            // Success cases
            assert_eq!(from_bytes::<$t>(b"0"), Ok(0));
            assert_eq!(from_bytes::<$t>(b"1"), Ok(1));
            assert_eq!(from_bytes::<$t>(b"+0"), Ok(0));
            assert_eq!(from_bytes::<$t>(b"-0"), Ok(0));
            assert_eq!(from_bytes::<$t>(b"+1"), Ok(1));
            assert_eq!(from_bytes::<$t>(b"-1"), Ok(-1));
            assert_eq!(from_bytes::<$t>(b"+9223372036854775807"), Ok(std::$i::MAX));
            assert_eq!(from_bytes::<$t>(b"-9223372036854775808"), Ok(std::$i::MIN));

            // Error cases
            assert_eq!(from_bytes::<$t>(b""), Err(ParseIntError));
            assert_eq!(from_bytes::<$t>(b"x"), Err(ParseIntError));

            // The final mul by the radix will overflow
            assert_eq!(
                from_bytes::<$t>(b"92233720368547758071"),
                Err(ParseIntError)
            );

            // The adding the final 8 digit will overflow
            assert_eq!(from_bytes::<$t>(b"9223372036854775808"), Err(ParseIntError));

            fn expect_std_error(_: impl std::error::Error) {}
            expect_std_error(ParseIntError);
        };
    }

    #[test]
    fn test_from_bytes_i64() {
        test_for_type!(i64, i64);
    }

    #[test]
    fn test_from_bytes_isize() {
        test_for_type!(isize, isize);
    }
}
