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
//! assert_eq!(Currency::JPY.exponent(), Some(0));
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]

pub use iso_country::Country;

#[cfg(feature = "with-serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "with-serde")))]
use serde::{Deserialize, Serialize};

#[cfg(feature = "iterator")]
#[cfg_attr(docsrs, doc(cfg(feature = "iterator")))]
use strum::EnumIter;
#[cfg(feature = "iterator")]
#[cfg_attr(docsrs, doc(cfg(feature = "iterator")))]
pub use strum::IntoEnumIterator;

include!(concat!(env!("OUT_DIR"), "/isodata.rs"));

#[derive(PartialEq, Eq)]
pub struct CurrencySymbol {
    pub symbol: String,
    pub subunit_symbol: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
            None => Err(ParseCurrencyError),
        }
    }
}

/// Extra information for a currency
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Flag {
    /// The currency is a fund
    Fund,
    /// The currency is a special currency
    Special,
    /// The currency is superseded by another currency
    Superseded(Currency),
}

#[cfg(test)]
mod tests {
    use crate::{Country, Currency, ParseCurrencyError, Flag};

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
        assert_eq!(Currency::CHF.used_by(), vec![Country::LI, Country::CH]);
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
        /* [Malagasy ariary](https://en.wikipedia.org/wiki/Malagasy_ariary) (`MRU`)
        and the [Mauritanian ouguiya](https://en.wikipedia.org/wiki/Mauritanian_ouguiya) (`MGA`)
        are technically divided into 5 subunits (iraimbilanja and khoum).
        However, while they have a face value of "1/5" and are referred to as a "fifth" (Khoum/cinquième),
        these are not used in practice. When written out, a single significant digit is used (example: 1.2 UM so that 10 UM = 1 MRU).
        -- Source [Wikipedia](https://en.wikipedia.org/wiki/ISO_4217#cite_note-divby5-15). */
        assert_eq!(Currency::MRU.subunit_fraction(), Some(100));
        assert_eq!(Currency::XAU.subunit_fraction(), None);
    }

    #[test]
    fn subunit_exponent() {
        assert_eq!(Currency::EUR.exponent(), Some(2));
        assert_eq!(Currency::JPY.exponent(), Some(0));
        assert_eq!(Currency::MRU.exponent(), Some(2));
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

        assert_eq!(
            serde_json::to_string(&hashmap).unwrap(),
            "{\"foo\":\"EUR\"}"
        );
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

    #[test]
    #[cfg(feature = "iterator")]
    fn test_iterator() {
        use crate::IntoEnumIterator;
        let mut iter = Currency::iter();
        assert_eq!(iter.next(), Some(Currency::AED));
        assert_eq!(iter.next(), Some(Currency::AFN));
    }

    #[test]
    fn test_is_fund() {
        assert!(Currency::BOV.is_fund());
        assert!(!Currency::EUR.is_fund());
    }

    #[test]
    fn test_is_special() {
        assert!(Currency::XBA.is_special());
        assert!(!Currency::EUR.is_special());
    }

    #[test]
    fn test_is_superseded() {
        assert_eq!(Currency::VED.is_superseded(), Some(Currency::VES));
        assert_eq!(Currency::VES.is_superseded(), None);
    }

    #[test]
    fn test_latest() {
        assert_eq!(Currency::VED.latest(), Currency::VES);
        assert_eq!(Currency::VES.latest(), Currency::VES);
    }

    #[test]
    fn test_flags() {
        assert_eq!(Currency::BOV.flags(), vec![Flag::Fund]);
        assert_eq!(Currency::XBA.flags(), vec![Flag::Special]);
        assert_eq!(Currency::VED.flags(), vec![Flag::Superseded(Currency::VES)]);
        assert_eq!(Currency::VES.flags(), vec![]);
    }

    #[test]
    fn test_has_flag() {
        assert!(Currency::BOV.has_flag(Flag::Fund));
        assert!(!Currency::XBA.has_flag(Flag::Fund));
    }

    #[test]
    fn test_from_country() {
        assert_eq!(Currency::from_country(Country::AF), vec![Currency::AFN]);
        assert_eq!(Currency::from_country(Country::IO), vec![Currency::GBP, Currency::USD]);
    }
}
