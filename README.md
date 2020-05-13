# ISO 4217 currency codes

[![](https://docs.rs/iso_currency/badge.svg)](https://docs.rs/iso_currency)
![](https://github.com/zbrox/iso_currency/workflows/Build/badge.svg)
![](https://img.shields.io/crates/v/iso_currency.svg)

This crate provides an enum that represents all ISO 4217 currencies and 
has simple methods to convert between numeric and character code, list of 
territories where each currency is used, the symbol, and the English name of the currency.

The data for this is taken from 
[https://en.wikipedia.org/wiki/ISO_4217](https://en.wikipedia.org/wiki/ISO_4217)

## Features

The crate has only one optional feature - `with-serde`. If you need serialization/deserialization support using `serde` you should include the feature in your dependency on `iso_currency`, for example like this:

```toml
iso_currency = { version = "0.3.0", features = ["with-serde"] }
```

## Examples

```rust
use iso_currency::Currency;

assert_eq!(Currency::EUR.name(), "Euro");
assert_eq!(Currency::EUR.numeric(), 978);
assert_eq!(Currency::from_numeric(978), Some(Currency::EUR));
assert_eq!(Currency::from_code("EUR"), Some(Currency::EUR));
assert_eq!(Currency::CHF.used_by(), vec!["Liechtenstein", "Switzerland"]);
assert_eq!(format!("{}", Currency::EUR.symbol()), "€");
```
