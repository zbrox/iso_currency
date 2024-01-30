use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

// use Tab separated so we can easily split on a rarely used character
static TSV_TABLE_PATH: &str = "isodata.tsv";

struct IsoData {
    alpha3: String,
    numeric: u16,
    name: String,
    symbol: String,
    used_by: Option<Vec<String>>,
    subunit_symbol: Option<String>,
    exponent: Option<u16>,
}

fn read_table() -> Vec<IsoData> {
    let reader =
        BufReader::new(File::open(TSV_TABLE_PATH).expect("Couldn't read currency data table"));

    reader
        .lines()
        .skip(1)
        .map(|line| {
            let line = line.expect("Problems reading line from ISO data CSV file");

            let columns: Vec<&str> = line.split('\t').collect();

            IsoData {
                alpha3: columns[0].into(),
                numeric: columns[1].parse::<u16>().unwrap_or_else(|_| {
                    panic!("Could not parse numeric code to u16 for {}", &columns[0])
                }),
                name: columns[2].into(),
                used_by: match columns[3].is_empty() {
                    true => None,
                    false => Some(
                        columns[3]
                            .split(';')
                            .map(|v| v.to_owned())
                            .collect::<Vec<String>>(),
                    ),
                },
                symbol: columns[4].into(),
                subunit_symbol: match columns[5].is_empty() {
                    true => None,
                    false => Some(columns[5].into()),
                },
                exponent: match columns[6].is_empty() {
                    true => None,
                    false => Some(columns[6].parse::<u16>().unwrap_or_else(|_| {
                        panic!("Could not parse exponent to u16 for {:?}", &columns[0])
                    })),
                },
            }
        })
        .collect()
}

fn write_enum(file: &mut BufWriter<File>, data: &[IsoData]) {
    let body: TokenStream = data
        .iter()
        .map(|currency| {
            let currency_name = format!("{}", &currency.name);
            let variant = Ident::new(&currency.alpha3, Span::call_site());
            quote! {
                #[doc = #currency_name]
                #variant,
            }
        })
        .collect();
    let outline = quote! {
        #[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
        #[cfg_attr(feature = "iterator", derive(EnumIter))]
        #[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub enum Currency {
            #body
        }
    };

    write!(file, "{}", outline.to_string()).unwrap();
}

fn generate_numeric_method(data: &[IsoData]) -> TokenStream {
    let match_arms: TokenStream = data
        .iter()
        .map(|currency| {
            let variant = Ident::new(&currency.alpha3, Span::call_site());
            let numeric = currency.numeric;
            quote! {
                Currency::#variant => #numeric,
            }
        })
        .collect();
    quote! {
        /// Returns the numeric code of the currency
        ///
        /// This method will return the ISO 4217 numeric code of the currency
        ///
        /// # Example
        ///
        /// ```
        /// use iso_currency::Currency;
        ///
        /// assert_eq!(Currency::EUR.numeric(), 978);
        /// ```
        pub fn numeric(self) -> u16 {
            match self {
                #match_arms
            }
        }
    }
}

fn name_method(data: &[IsoData]) -> TokenStream {
    let match_arms: TokenStream = data
        .iter()
        .map(|currency| {
            let variant = Ident::new(&currency.alpha3, Span::call_site());
            let name = currency.name.as_str();
            quote! {
                Currency::#variant => #name,
            }
        })
        .collect();
    quote! {
        /// Returns the name of the currency in English
        ///
        /// This method will return the English name of the currency
        ///
        /// # Example
        ///
        /// ```
        /// use iso_currency::Currency;
        ///
        /// assert_eq!(Currency::EUR.name(), "Euro");
        /// ```
        pub fn name(&self) -> &str {
            match self {
                #match_arms
            }
        }
    }
}

fn code_method(data: &[IsoData]) -> TokenStream {
    let match_arms: TokenStream = data
        .iter()
        .map(|currency| {
            let variant = Ident::new(&currency.alpha3, Span::call_site());
            let code = currency.alpha3.as_str();
            quote! {
                Currency::#variant => #code,
            }
        })
        .collect();
    quote! {
        /// Returns the ISO 4217 code
        ///
        /// # Example
        ///
        /// ```
        /// use iso_currency::Currency;
        ///
        /// assert_eq!(Currency::EUR.code(), "EUR");
        /// ```
        pub fn code(&self) -> &'static str {
            match self {
                #match_arms
            }
        }
    }
}

fn used_by_method(data: &[IsoData]) -> TokenStream {
    let match_arms: TokenStream = data
        .iter()
        .map(|currency| {
            let country_list: TokenStream = match &currency.used_by {
                Some(v) => v
                    .iter()
                    .map(|c| {
                        let country_ident = Ident::new(&c, Span::call_site());
                        quote!(Country::#country_ident,)
                    })
                    .collect(),
                None => quote!(),
            };
            let variant = Ident::new(&currency.alpha3, Span::call_site());
            quote! {
                Currency::#variant => vec![#country_list],
            }
        })
        .collect();
    quote! {
        /// Returns a list of locations which use the currency
        ///
        /// This method will return a list of locations which use the currency.
        /// The use is non-exclusive, so it might mean that the location is using
        /// other currencies as well. The list of locations is sorted.
        ///
        /// # Example
        ///
        /// ```
        /// use iso_currency::{Currency, Country};
        ///
        /// assert_eq!(
        ///     Currency::CHF.used_by(),
        ///     vec![Country::LI, Country::CH]
        /// );
        /// ```
        pub fn used_by(self) -> Vec<Country> {
            let mut territories = match self {
                #match_arms
            };
            territories.sort();
            territories
        }
    }
}

fn symbol_method(data: &[IsoData]) -> TokenStream {
    let match_arms: TokenStream = data
        .iter()
        .map(|currency| {
            let variant = Ident::new(&currency.alpha3, Span::call_site());
            let symbol = currency.symbol.as_str();
            let subunit_symbol = match currency.subunit_symbol {
                Some(ref v) => quote!(Some(#v)),
                None => quote!(None),
            };
            quote! {
                Currency::#variant => CurrencySymbol::new(#symbol, #subunit_symbol),
            }
        })
        .collect();
    quote! (
        /// Returns the currency's symbol
        ///
        /// This method will return the symbol commonly used to represent the
        /// currency. In case there is no symbol associated the international
        /// currency symbol will be returned.
        ///
        /// # Example
        ///
        /// ```
        /// use iso_currency::Currency;
        ///
        /// assert_eq!(format!("{}", Currency::EUR.symbol()), "€");
        /// assert_eq!(format!("{}", Currency::XXX.symbol()), "¤");
        /// ```
        pub fn symbol(self) -> CurrencySymbol {
            match self {
                #match_arms
            }
        }
    )
}

fn from_code_method(data: &[IsoData]) -> TokenStream {
    let match_arms: TokenStream = data
        .iter()
        .map(|currency| {
            let code = currency.alpha3.as_str();
            let variant = Ident::new(&currency.alpha3, Span::call_site());
            quote! {
                #code => Some(Currency::#variant),
            }
        })
        .collect();
    quote!(
        /// Create a currency instance from a ISO 4217 character code
        ///
        /// # Example
        ///
        /// ```
        /// use iso_currency::Currency;
        ///
        /// assert_eq!(Currency::from_code("EUR"), Some(Currency::EUR));
        /// ```
        pub fn from_code(code: &str) -> Option<Currency> {
            if code.len() != 3 {
                return None;
            }
            match code {
                #match_arms
                _ => None,
            }
        }
    )
}

fn from_numeric_method(data: &[IsoData]) -> TokenStream {
    let match_arms: TokenStream = data
        .iter()
        .map(|currency| {
            let numeric_code = currency.numeric;
            let variant = Ident::new(&currency.alpha3, Span::call_site());
            quote! {
                #numeric_code => Some(Currency::#variant),
            }
        })
        .collect();
    quote!(
        /// Create a currency instance from a ISO 4217 numeric code
        ///
        /// # Example
        ///
        /// ```
        /// use iso_currency::Currency;
        ///
        /// assert_eq!(Currency::from_numeric(978), Some(Currency::EUR));
        /// ```
        pub fn from_numeric(numeric_code: u16) -> Option<Currency> {
            match numeric_code {
                #match_arms
                _ => None,
            }
        }
    )
}

fn exponent_method(data: &[IsoData]) -> TokenStream {
    let match_arms: TokenStream = data
        .iter()
        .filter(|c| c.exponent.is_some())
        .map(|currency| {
            let variant = Ident::new(&currency.alpha3, Span::call_site());
            let exponent = match currency.exponent {
                Some(v) => quote!(Some(#v)),
                None => quote!(None),
            };
            quote! {
                Currency::#variant => #exponent,
            }
        })
        .collect();
    quote!(
        /// Returns the exponent of a currency (number of decimal places)
        /// For example, 1.00 Euro a 2 subunits so this will return Some(2) for EUR.
        ///
        /// This returns an optional value because some currencies don't have a subunit.
        ///
        /// # Example
        ///
        /// ```
        /// use iso_currency::Currency;
        ///
        /// assert_eq!(Currency::EUR.exponent(), Some(2));
        /// assert_eq!(Currency::JPY.exponent(), Some(0));
        /// ```
        pub fn exponent(self) -> Option<u16> {
            match self {
                #match_arms
                _ => None,
            }
        }
    )
}

fn subunit_fraction_method(data: &[IsoData]) -> TokenStream {
    let match_arms: TokenStream = data
        .iter()
        .filter(|c| c.exponent.is_some())
        .map(|currency| {
            let variant = Ident::new(&currency.alpha3, Span::call_site());
            let value = match &currency.exponent {
                Some(v) => quote!(Some(10_u16.pow(#v as u32))),
                None => quote!(None),
            };
            quote! {
                Currency::#variant => #value,
            }
        })
        .collect();
    quote!(
        /// Returns how many of the subunits equal the main unit of the currency
        /// For example there are a 100 cents in 1 Euro so this will return Some(100) for EUR.
        ///
        /// This returns an optional value because some currencies don't have a subunit.
        ///
        /// # Example
        ///
        /// ```
        /// use iso_currency::Currency;
        ///
        /// assert_eq!(Currency::EUR.subunit_fraction(), Some(100));
        /// ```
        pub fn subunit_fraction(self) -> Option<u16> {
            match self {
                #match_arms
                _ => None,
            }
        }
    )
}

fn write_enum_impl(file: &mut BufWriter<File>, data: &[IsoData]) {
    let numeric_method = generate_numeric_method(data);
    let name_method = name_method(data);
    let code_method = code_method(data);
    let used_by_method = used_by_method(data);
    let symbol_method = symbol_method(data);
    let from_code_method = from_code_method(data);
    let from_numeric_method = from_numeric_method(data);
    let exponent_method = exponent_method(data);
    let subunit_fraction_method = subunit_fraction_method(data);

    let outline = quote! (
      impl Currency {
          #numeric_method

          #name_method

          #code_method

          #used_by_method

          #symbol_method

          #from_code_method

          #from_numeric_method

          #exponent_method

          #subunit_fraction_method
      }
    );

    write!(file, "{}", outline.to_string()).unwrap();
}

fn main() {
    let out_path = Path::new(&env::var("OUT_DIR").unwrap()).join("isodata.rs");

    let isodata = read_table();

    {
        let mut file =
            BufWriter::new(File::create(out_path).expect("Couldn't write to output file"));
        write_enum(&mut file, &isodata);
        write_enum_impl(&mut file, &isodata);
    }
}
