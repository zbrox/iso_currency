# ISO 4217 currency codes

[![docs badge](https://docs.rs/iso_currency/badge.svg)](https://docs.rs/iso_currency)
[![build badge](https://github.com/zbrox/iso_currency/workflows/Build/badge.svg)](https://github.com/zbrox/iso_currency/actions/workflows/build.yml)
[![crates badge](https://img.shields.io/crates/v/iso_currency.svg)](https://crates.io/crates/iso_currency)
![license badge](https://img.shields.io/crates/l/iso_currency.svg)

This crate provides an enum that represents all ISO 4217 currencies and
has simple methods to convert between numeric and character code, list of
territories where each currency is used, the symbol, and the English name of the currency.

The data for this is taken from
[https://en.wikipedia.org/wiki/ISO_4217](https://en.wikipedia.org/wiki/ISO_4217)

The `Country` enum is re-exported from the only dependency - the [iso_country](https://crates.io/crates/iso_country) crate.

## Features

The crate has some optional features:

- `with-serde`
- `iterator`
- `with-schemars`
- `with-sqlx-sqlite`
- `with-sqlx-postgres`
- `with-sqlx-mysql`

### with-serde

If you need serialization/deserialization support using `serde` you should include the feature in your dependency on `iso_currency`.

This would derive serde's `Serialize` and `Deserialize` on `Currency`.

### iterator

If you specify the `iterator` feature on `iso_currency`, it will derive [strum's](https://crates.io/crates/strum) `EnumIter` trait on `Currency`, which provides an iterator over all variants of it. Here's an example usage:

```rust
use iso_currency::IntoEnumIterator;
let mut iter = Currency::iter();
```

### with-schemars

If you need to generate a JSON schema for your project, you can use the `with-schemars` feature. This will derive [`schemars's`](https://crates.io/crates/schemars) `JsonSchema` trait on `Currency`.

**NOTE**: This feature enables `with-serde` as well.

### with-sqlx-sqlite

Implements the `Type` and `Decode` traits from [sqlx](https://github.com/launchbadge/sqlx) version >0.7 for SQLite on the `Currency` struct.

### with-sqlx-postgres

Implements the `Type` and `Decode` traits from [sqlx](https://github.com/launchbadge/sqlx) version >0.7 for PostgreSQL on the `Currency` struct.

### with-sqlx-mysql

Implements the `Type` and `Decode` traits from [sqlx](https://github.com/launchbadge/sqlx) version >0.7 for MySQL on the `Currency` struct.

## Examples

```rust
use iso_currency::{Currency, Country};

assert_eq!(Currency::EUR.name(), "Euro");
assert_eq!(Currency::EUR.numeric(), 978);
assert_eq!(Currency::from_numeric(978), Some(Currency::EUR));
assert_eq!(Currency::from_code("EUR"), Some(Currency::EUR));
assert_eq!(Currency::CHF.used_by(), vec![Country::LI, Country::CH]);
assert_eq!(format!("{}", Currency::EUR.symbol()), "€");
assert_eq!(Currency::EUR.subunit_fraction(), Some(100));
assert_eq!(Currency::JPY.exponent(), Some(0));
```

## Want to help improve the data?

The `Currency` enum and its implementations are generated from the `isodata.tsv` file. It is a table of `<tab>` separated values. If you wanna correct some value or add some missing values you just need to make a pull request editing that table.

One thing to watch out for is to have always the same amount of fields on a row, even if an optional field is missing. This means on each row you should have **6** tabs.

The `used_by_alpha2` column is a bit different. It can be empty but if not it includes a list, separated by a semicolon (without a trailing semicolon), of `ISO 3166-1` 2-letter country codes in all caps.
