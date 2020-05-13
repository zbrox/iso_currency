# ISO 4217 currency codes

[![](https://docs.rs/iso_currency/badge.svg)](https://docs.rs/iso_currency)
![](https://github.com/zbrox/iso_currency/workflows/Build/badge.svg)
![](https://img.shields.io/crates/v/iso_currency.svg)

This crate provides an enum that represents all ISO 4217 currencies and 
has simple methods to convert between numeric and character code, list of 
territories where each currency is used, the symbol, and the English name of the currency.

The data for this is taken from 
[https://en.wikipedia.org/wiki/ISO_4217](https://en.wikipedia.org/wiki/ISO_4217)

The `Country` enum is re-exported from the only dependency - the [iso_country](https://crates.io/crates/iso_country) crate.

## Features

The crate has only one optional feature - `with-serde`. If you need serialization/deserialization support using `serde` you should include the feature in your dependency on `iso_currency`, for example like this:

```toml
iso_currency = { version = "0.3.2", features = ["with-serde"] }
```

## Examples

```rust
use iso_currency::{Currency, Country};

assert_eq!(Currency::EUR.name(), "Euro");
assert_eq!(Currency::EUR.numeric(), 978);
assert_eq!(Currency::from_numeric(978), Some(Currency::EUR));
assert_eq!(Currency::from_code("EUR"), Some(Currency::EUR));
assert_eq!(Currency::CHF.used_by(), vec![Country::LI, Country::CH]);
assert_eq!(format!("{}", Currency::EUR.symbol()), "€");
```

## Want to help improve the data?

The `Currency` enum and its implementations are generated from the `isodata.tsv` file. It is a table of `<tab>` separated values. If you wanna correct some value or add some missing values you just need to make a pull request editing that table.

One thing to watch out for is to have always the same amount of fields on a row, even if an optional field is missing. This means on each row you should have **5** tabs.

The `used_by_alpha2` column is a bit different. It can be empty but if not it includes a list, separated by a semicolon (without a trailing semicolon), of `ISO 3166-1` 2-letter country codes in all caps.
