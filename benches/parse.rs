use std::str::FromStr;

fn main() {
    // Run registered benchmarks.
    divan::main();
}

#[divan::bench]
fn parse_code() {
    iso_currency::Currency::from_code("EUR").unwrap();
}

#[divan::bench]
fn parse_numeric() {
    iso_currency::Currency::from_numeric(978).unwrap();
}

#[divan::bench]
fn from_str() {
    iso_currency::Currency::from_str("SEK").unwrap();
}
