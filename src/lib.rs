//! ISO 4217 currency codes
//!
//! This crate provides an enum that represents all ISO 4217 currencies and
//! has simple methods to convert between numeric and character code, list of
//! territories where each currency is used, the symbol,
//! and the English name of the currency.
//!
//! The data for this is taken from
//! [https://en.wikipedia.org/wiki/ISO_4217](https://en.wikipedia.org/wiki/ISO_4217)
//!
//! The `Country` enum is re-exported from the only dependency - the [iso_country](https://crates.io/crates/iso_country) crate.
//!
//! # Examples
//!
//! ```
//! use iso_currency::{Currency, Country};
//!
//! assert_eq!(Currency::EUR.name(), "Euro");
//! assert_eq!(Currency::EUR.numeric(), 978);
//! assert_eq!(Currency::from_numeric(978), Some(Currency::EUR));
//! assert_eq!(Currency::from_code("EUR"), Some(Currency::EUR));
//! assert_eq!(Currency::CHF.used_by(), vec![Country::LI, Country::CH]);
//! assert_eq!(format!("{}", Currency::EUR.symbol()), "€");
//! assert_eq!(Currency::EUR.subunit_fraction(), Some(100));
//! ```

pub use iso_country::Country;

#[cfg(feature = "with-serde")]
use serde::{Deserialize, Serialize};

include!(concat!(env!("OUT_DIR"), "/isodata.rs"));

#[derive(PartialEq)]
pub struct CurrencySymbol {
    pub symbol: String,
    pub subunit_symbol: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseCurrencyError;

impl std::fmt::Display for ParseCurrencyError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "not a valid ISO 4217 currency code")
    }
}

impl std::fmt::Debug for CurrencySymbol {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.symbol)
    }
}

impl std::fmt::Display for CurrencySymbol {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.symbol)
    }
}

impl CurrencySymbol {
    /// Represents the commonly used symbol for a currency
    ///
    /// Data for the symbols was collected from
    /// [https://en.wikipedia.org/wiki/Currency_symbol#List_of_presently-circulating_currency_symbols]()
    ///
    /// TODO: Add data about subunit symbols for every currency
    /// TODO: Add data about English representations of some currency symbols
    /// TODO: Maybe add data about alternative variants of the symbols
    /// TODO: Add data about position of symbol (according to locale) when formatting a sum of money
    ///
    pub fn new(symbol: &str, subunit_symbol: Option<&str>) -> CurrencySymbol {
        CurrencySymbol {
            symbol: symbol.to_owned(),
            subunit_symbol: subunit_symbol.map(|v| v.to_owned()),
        }
    }
}

impl std::fmt::Debug for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.code())
    }
}

impl std::fmt::Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl std::str::FromStr for Currency {
    type Err = ParseCurrencyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match Self::from_code(s) {
            Some(c) => Ok(c),
            None => Err(ParseCurrencyError)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Country, Currency, ParseCurrencyError};

    #[cfg(feature = "with-serde")]
    use serde_json;
    #[cfg(feature = "with-serde")]
    use std::collections::HashMap;

    #[test]
    fn return_numeric_code() {
        assert_eq!(Currency::EUR.numeric(), 978);
        assert_eq!(Currency::BBD.numeric(), 52);
        assert_eq!(Currency::XXX.numeric(), 999);
    }

    #[test]
    fn return_name() {
        assert_eq!(Currency::EUR.name(), "Euro");
        assert_eq!(Currency::BGN.name(), "Bulgarian lev");
        assert_eq!(Currency::USD.name(), "United States dollar");
    }

    #[test]
    fn return_code() {
        assert_eq!(Currency::EUR.code(), "EUR");
    }

    #[test]
    fn from_code() {
        assert_eq!(Currency::from_code("EUR"), Some(Currency::EUR));
        assert_eq!(Currency::from_code("SEK"), Some(Currency::SEK));
        assert_eq!(Currency::from_code("BGN"), Some(Currency::BGN));
        assert_eq!(Currency::from_code("AAA"), None);
    }

    #[test]
    #[allow(clippy::zero_prefixed_literal)]
    fn from_numeric() {
        assert_eq!(Currency::from_numeric(999), Some(Currency::XXX));
        assert_eq!(Currency::from_numeric(052), Some(Currency::BBD));
        assert_eq!(Currency::from_numeric(978), Some(Currency::EUR));
        assert_eq!(Currency::from_numeric(012), Some(Currency::DZD));
        assert_eq!(Currency::from_numeric(123), None);
    }

    #[test]
    fn used_by() {
        assert_eq!(Currency::BGN.used_by(), vec![Country::BG]);
        assert_eq!(
            Currency::CHF.used_by(),
            vec![Country::LI, Country::CH]
        );
    }

    #[test]
    fn symbol() {
        assert_eq!(format!("{}", Currency::EUR.symbol()), "€");
        assert_eq!(format!("{}", Currency::XXX.symbol()), "¤");
        assert_eq!(format!("{}", Currency::GEL.symbol()), "ლ");
        assert_eq!(format!("{}", Currency::AED.symbol()), "د.إ");
    }

    #[test]
    fn subunit_fraction() {
        assert_eq!(Currency::EUR.subunit_fraction(), Some(100));
        assert_eq!(Currency::DZD.subunit_fraction(), Some(100));
        assert_eq!(Currency::MRU.subunit_fraction(), Some(5));
        assert_eq!(Currency::XAU.subunit_fraction(), None);
    }

    #[test]
    #[cfg(feature = "with-serde")]
    fn deserialize() {
        let hashmap: HashMap<&str, Currency> = serde_json::from_str("{\"foo\": \"EUR\"}").unwrap();
        assert_eq!(hashmap["foo"], Currency::EUR);
    }

    #[test]
    #[cfg(feature = "with-serde")]
    fn serialize() {
        let mut hashmap: HashMap<&str, Currency> = HashMap::new();
        hashmap.insert("foo", Currency::EUR);

        assert_eq!(serde_json::to_string(&hashmap).unwrap(), "{\"foo\":\"EUR\"}");
    }

    #[test]
    fn can_be_sorted() {
        let mut v = vec![Currency::SEK, Currency::DKK, Currency::EUR];
        v.sort();
        assert_eq!(v, vec![Currency::DKK, Currency::EUR, Currency::SEK]);
    }
    
    #[test]
    fn implements_from_str() {
        use std::str::FromStr;
        assert_eq!(Currency::from_str("EUR"), Ok(Currency::EUR));
        assert_eq!(Currency::from_str("SEK"), Ok(Currency::SEK));
        assert_eq!(Currency::from_str("BGN"), Ok(Currency::BGN));
        assert_eq!(Currency::from_str("AAA"), Err(ParseCurrencyError));
    }
}
