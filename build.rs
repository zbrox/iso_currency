use std::{
    collections::HashMap,
    env,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::Path,
};

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
    is_special: bool,
    is_fund: bool,
    is_superseded: Option<String>,
}

fn parse_superseded(flag: &str) -> Option<String> {
    let mut superseded = None;
    if flag.starts_with("superseded") {
        superseded = Some(
            flag.split(&['(', ')'])
                .nth(1)
                .expect("Invalid format for superseded flag")
                .to_string(),
        );
    }
    superseded
}

fn parse_flags(flags: &str) -> (bool, bool, Option<String>) {
    let mut is_special = false;
    let mut is_fund = false;
    let mut is_superseded = None;

    for flag in flags.split(',') {
        match flag {
            "special" => is_special = true,
            "fund" => is_fund = true,
            // example superseded(USD)
            _ => is_superseded = parse_superseded(flag),
        }
    }

    (is_special, is_fund, is_superseded)
}

fn flags_vec(data: &IsoData) -> TokenStream {
    let mut flags = Vec::new();
    if data.is_special {
        flags.push(quote!(Flag::Special));
    }
    if data.is_fund {
        flags.push(quote!(Flag::Fund));
    }
    if let Some(superseded) = &data.is_superseded {
        let currency = Ident::new(superseded, Span::call_site());
        flags.push(quote!(Flag::Superseded(Currency::#currency)));
    }
    quote!(vec![#(#flags),*])
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
            let flags = parse_flags(columns[7]);

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
                is_special: flags.0,
                is_fund: flags.1,
                is_superseded: flags.2,
            }
        })
        .collect()
}

fn write_enum(file: &mut BufWriter<File>, data: &[IsoData]) {
    let body: TokenStream = data
        .iter()
        .map(|currency| {
            let currency_name = currency.name.as_str();
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
        #[cfg_attr(feature = "with-schemars", derive(JsonSchema))]
        #[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub enum Currency {
            #body
        }
    };

    write!(file, "{}", outline).unwrap();
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
                        let country_ident = Ident::new(c, Span::call_site());
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
            #[allow(clippy::match_single_binding)]
            match code {
                #match_arms
                #[allow(unreachable_patterns)]
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
            #[allow(clippy::match_single_binding)]
            match numeric_code {
                #match_arms
                #[allow(unreachable_patterns)]
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
            #[allow(clippy::match_single_binding)]
            match self {
                #match_arms
                #[allow(unreachable_patterns)]
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
            #[allow(clippy::match_single_binding)]
            match self {
                #match_arms
                #[allow(unreachable_patterns)]
                _ => None,
            }
        }
    )
}

fn joint_match_currency_bool(data: &[&IsoData], value: bool) -> TokenStream {
    let list: Vec<_> = data
        .iter()
        .map(|currency| {
            let variant = Ident::new(&currency.alpha3, Span::call_site());
            quote! {
                Currency::#variant
            }
        })
        .collect();

    quote!(
        #(#list)|* => #value,
    )
}

fn is_fund_method(data: &[IsoData]) -> TokenStream {
    let partitions: (Vec<_>, Vec<_>) = data.iter().partition(|c| c.is_fund);
    let left_match_arms = if !partitions.0.is_empty() {
        joint_match_currency_bool(
            partitions.0.as_slice(),
            partitions.0.first().unwrap().is_fund,
        )
    } else {
        quote!()
    };
    let right_match_arms = if !partitions.1.is_empty() {
        joint_match_currency_bool(
            partitions.1.as_slice(),
            partitions.1.first().unwrap().is_fund,
        )
    } else {
        quote!()
    };

    quote!(
        /// Returns true if the currency is a fund
        pub fn is_fund(self) -> bool {
            match self {
                #left_match_arms
                #right_match_arms
            }
        }
    )
}

fn is_special_method(data: &[IsoData]) -> TokenStream {
    let partitions: (Vec<_>, Vec<_>) = data.iter().partition(|c| c.is_special);
    let left_match_arms = if !partitions.0.is_empty() {
        joint_match_currency_bool(
            partitions.0.as_slice(),
            partitions.0.first().unwrap().is_special,
        )
    } else {
        quote!()
    };
    let right_match_arms = if !partitions.1.is_empty() {
        joint_match_currency_bool(
            partitions.1.as_slice(),
            partitions.1.first().unwrap().is_special,
        )
    } else {
        quote!()
    };

    quote!(
        /// Returns true if the currency is a special currency
        ///
        /// Example of special currencies are gold, silver, the IMF's
        /// Special Drawing Rights (SDRs).
        pub fn is_special(self) -> bool {
            match self {
                #left_match_arms
                #right_match_arms
            }
        }
    )
}

fn is_superseded_method(data: &[IsoData]) -> TokenStream {
    let match_arms: TokenStream = data
        .iter()
        .filter(|c| c.is_superseded.is_some())
        .map(|currency| {
            let variant = Ident::new(&currency.alpha3, Span::call_site());
            let value = match &currency.is_superseded {
                Some(v) => {
                    let v = Ident::new(v, Span::call_site());
                    quote!(Some(Currency::#v))
                }
                None => quote!(None),
            };
            quote! {
                Currency::#variant => #value,
            }
        })
        .collect();
    quote!(
        /// Returns the currency that superseded this currency
        ///
        /// In case the currency is not superseded by another it will return `None`
        pub fn is_superseded(self) -> Option<Self> {
            #[allow(clippy::match_single_binding)]
            match self {
                #match_arms
                #[allow(unreachable_patterns)]
                _ => None
            }
        }
    )
}

fn latest_method(data: &[IsoData]) -> TokenStream {
    let match_arms: TokenStream = data
        .iter()
        .map(|currency| {
            let variant = Ident::new(&currency.alpha3, Span::call_site());
            let value = match currency.is_superseded {
                Some(ref v) => {
                    let v = Ident::new(v, Span::call_site());
                    quote!(Currency::#v)
                }
                None => quote!(Currency::#variant),
            };

            quote! {
                Currency::#variant => #value,
            }
        })
        .collect();
    quote!(
        /// Returns either the currency itself or what superseded it
        ///
        /// In case the currency is not superseded by another it will return itself.
        /// Currently the data doesn't include any currency which has been superseded
        /// by another currency which in turn has been superseded by another currency.
        /// Therefore this doesn't follow a chain of currencies but is just
        /// a convenience method with a slightly different signature than `Currency::is_superseded`.
        pub fn latest(self) -> Self {
            match self {
                #match_arms
            }
        }
    )
}

fn flags_method(isodata: &[IsoData]) -> TokenStream {
    let match_arms: TokenStream = isodata
        .iter()
        .map(|currency| {
            let variant = Ident::new(&currency.alpha3, Span::call_site());
            let flags = flags_vec(currency);
            quote! {
                Currency::#variant => #flags,
            }
        })
        .collect();
    quote!(
        /// Returns a list of extra information flags about the currency"
        pub fn flags(self) -> Vec<Flag> {
            match self {
                #match_arms
            }
        }
    )
}

fn has_flag_method(data: &[IsoData]) -> TokenStream {
    let match_arms: TokenStream = data
        .iter()
        .map(|currency| {
            let variant = Ident::new(&currency.alpha3, Span::call_site());
            quote! {
                Currency::#variant => Currency::#variant.flags().contains(&flag),
            }
        })
        .collect();
    quote!(
        /// Returns true if the currency has the given flag
        pub fn has_flag(self, flag: Flag) -> bool {
            match self {
                #match_arms
            }
        }
    )
}

fn from_country_method(country_map: &HashMap<String, Vec<String>>) -> TokenStream {
    let match_arms: TokenStream = country_map
        .iter()
        .map(|(country, currencies)| {
            let country = Ident::new(country, Span::call_site());
            let currency_vec: TokenStream = currencies
                .iter()
                .map(|currency| Ident::new(currency, Span::call_site()))
                .map(|ident| quote!(Currency::#ident,))
                .collect();
            quote! {
                Country::#country => vec![#currency_vec],
            }
        })
        .collect();
    quote!(
        /// Returns a list of currencies used in a country
        pub fn from_country(country: Country) -> Vec<Self> {
            match country {
                #match_arms
                _ => vec![]
            }
        }
    )
}

fn write_enum_impl(
    file: &mut BufWriter<File>,
    data: &[IsoData],
    country_map: &HashMap<String, Vec<String>>,
) {
    let numeric_method = generate_numeric_method(data);
    let name_method = name_method(data);
    let code_method = code_method(data);
    let used_by_method = used_by_method(data);
    let symbol_method = symbol_method(data);
    let from_code_method = from_code_method(data);
    let from_numeric_method = from_numeric_method(data);
    let exponent_method = exponent_method(data);
    let subunit_fraction_method = subunit_fraction_method(data);
    let is_fund_method = is_fund_method(data);
    let is_special_method = is_special_method(data);
    let is_superseded_method = is_superseded_method(data);
    let latest_method = latest_method(data);
    let flags_method = flags_method(data);
    let has_flag_method = has_flag_method(data);
    let from_country_method = from_country_method(country_map);

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

          #is_fund_method

          #is_special_method

          #is_superseded_method

          #latest_method

          #flags_method

          #has_flag_method

          #from_country_method
      }
    );

    write!(file, "{}", outline).unwrap();
}

fn build_country_map(isodata: &[IsoData]) -> HashMap<String, Vec<String>> {
    let mut country_map = HashMap::new();
    for currency in isodata.iter() {
        if let Some(used_by) = &currency.used_by {
            for country in used_by.iter() {
                let country_list = country_map.entry(country.to_string()).or_insert(Vec::new());
                country_list.push(currency.alpha3.clone());
            }
        }
    }
    country_map
}

fn main() {
    println!("cargo:rerun-if-changed={TSV_TABLE_PATH}");

    let out_path = Path::new(&env::var("OUT_DIR").unwrap()).join("isodata.rs");

    let mut isodata = read_table();
    isodata.retain(|c| std::env::var(format!("CARGO_FEATURE_{}", c.alpha3.to_uppercase())).is_ok());

    if isodata.is_empty() {
        panic!("\n\x1b[31;1mNo currency has been enabled for iso_country.\x1b[0m\nYou should enable at least one currency in features.\n");
    }

    let country_map = build_country_map(&isodata);

    {
        let mut file =
            BufWriter::new(File::create(out_path).expect("Couldn't write to output file"));
        write_enum(&mut file, &isodata);
        write_enum_impl(&mut file, &isodata, &country_map);
    }
}
