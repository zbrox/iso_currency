use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

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
        superseded = Some(flag.split(&['(', ')']).nth(1).expect("Invalid format for superseded flag").to_string());
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

fn flags_vec(data: &IsoData) -> String {
    let mut flags = Vec::new();
    if data.is_special {
        flags.push("Flag::Special".to_string());
    }
    if data.is_fund {
        flags.push("Flag::Fund".to_string());
    }
    if let Some(superseded) = &data.is_superseded {
        flags.push(format!("Flag::Superseded(Currency::{})", superseded));
    }
    flags.join(",")
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
    writeln!(
        file,
        "#[cfg_attr(feature = \"with-serde\", derive(Serialize, Deserialize))]"
    )
    .unwrap();
    writeln!(
        file,
        "#[cfg_attr(feature = \"iterator\", derive(EnumIter))]"
    )
    .unwrap();
    writeln!(
        file,
        "#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]"
    )
    .unwrap();
    writeln!(file, "pub enum Currency {{").unwrap();

    for currency in data.iter() {
        writeln!(file, "    /// {}", &currency.name).unwrap();
        writeln!(file, "    {},", &currency.alpha3).unwrap();
    }

    writeln!(file, "}}").unwrap();
    writeln!(file).unwrap();
}

fn write_enum_impl(file: &mut BufWriter<File>, data: &[IsoData]) {
    writeln!(file, "impl Currency {{").unwrap();
    writeln!(file, "    /// Returns the numeric code of the currency").unwrap();
    writeln!(file, "    ///").unwrap();
    writeln!(
        file,
        "    /// This method will return the ISO 4217 numeric code of the currency"
    )
    .unwrap();
    writeln!(file, "    ///").unwrap();
    writeln!(file, "    /// # Example").unwrap();
    writeln!(file, "    ///").unwrap();
    writeln!(file, "    /// ```").unwrap();
    writeln!(file, "    /// use iso_currency::Currency;").unwrap();
    writeln!(file, "    ///").unwrap();
    writeln!(file, "    /// assert_eq!(Currency::EUR.numeric(), 978);").unwrap();
    writeln!(file, "    /// ```").unwrap();
    writeln!(file, "    pub fn numeric(self) -> u16 {{").unwrap();
    writeln!(file, "        match self {{").unwrap();
    for currency in data.iter() {
        writeln!(
            file,
            "            Currency::{} => {},",
            &currency.alpha3, currency.numeric
        )
        .unwrap();
    }
    writeln!(file, "        }}").unwrap();
    writeln!(file, "    }}").unwrap();
    writeln!(file).unwrap();

    writeln!(file, "    /// Returns the name of the currency in English").unwrap();
    writeln!(file, "    ///").unwrap();
    writeln!(
        file,
        "    /// This method will return the English name of the currency"
    )
    .unwrap();
    writeln!(file, "    ///").unwrap();
    writeln!(file, "    /// # Example").unwrap();
    writeln!(file, "    ///").unwrap();
    writeln!(file, "    /// ```").unwrap();
    writeln!(file, "    /// use iso_currency::Currency;").unwrap();
    writeln!(file, "    ///").unwrap();
    writeln!(file, "    /// assert_eq!(Currency::EUR.name(), \"Euro\");").unwrap();
    writeln!(file, "    /// ```").unwrap();
    writeln!(file, "    pub fn name(&self) -> &str {{").unwrap();
    writeln!(file, "        match self {{").unwrap();
    for currency in data.iter() {
        writeln!(
            file,
            "            Currency::{} => \"{}\",",
            &currency.alpha3, &currency.name
        )
        .unwrap();
    }
    writeln!(file, "        }}").unwrap();
    writeln!(file, "    }}").unwrap();
    writeln!(file).unwrap();

    writeln!(file, "    /// Returns the ISO 4217 code").unwrap();
    writeln!(file, "    ///").unwrap();
    writeln!(file, "    /// # Example").unwrap();
    writeln!(file, "    ///").unwrap();
    writeln!(file, "    /// ```").unwrap();
    writeln!(file, "    /// use iso_currency::Currency;").unwrap();
    writeln!(file, "    ///").unwrap();
    writeln!(file, "    /// assert_eq!(Currency::EUR.code(), \"EUR\");").unwrap();
    writeln!(file, "    /// ```").unwrap();
    writeln!(file, "    pub fn code(self) -> &'static str {{").unwrap();
    writeln!(file, "        match self {{").unwrap();
    for currency in data.iter() {
        writeln!(
            file,
            "            Currency::{} => \"{}\",",
            &currency.alpha3, &currency.alpha3
        )
        .unwrap();
    }
    writeln!(file, "        }}").unwrap();
    writeln!(file, "    }}").unwrap();
    writeln!(file).unwrap();

    writeln!(
        file,
        "    /// Returns a list of locations which use the currency"
    )
    .unwrap();
    writeln!(file, "    ///").unwrap();
    writeln!(
        file,
        "    /// This method will return a list of locations which use the currency."
    )
    .unwrap();
    writeln!(
        file,
        "    /// The use is non-exclusive, so it might mean that the location is using"
    )
    .unwrap();
    writeln!(
        file,
        "    /// other currencies as well. The list of locations is sorted."
    )
    .unwrap();
    writeln!(file, "    ///").unwrap();
    writeln!(file, "    /// # Example").unwrap();
    writeln!(file, "    ///").unwrap();
    writeln!(file, "    /// ```").unwrap();
    writeln!(file, "    /// use iso_currency::{{Currency, Country}};").unwrap();
    writeln!(file, "    /// ").unwrap();
    writeln!(file, "    /// assert_eq!(").unwrap();
    writeln!(file, "    ///     Currency::CHF.used_by(),").unwrap();
    writeln!(file, "    ///     vec![Country::LI, Country::CH]").unwrap();
    writeln!(file, "    /// );").unwrap();
    writeln!(file, "    /// ```").unwrap();
    writeln!(file, "    pub fn used_by(self) -> Vec<Country> {{").unwrap();
    writeln!(file, "        let mut territories = match self {{").unwrap();
    for currency in data.iter() {
        let country_list: String = match &currency.used_by {
            Some(v) => v.iter().map(|c| format!("Country::{},", c)).collect(),
            None => "".to_string(),
        };
        writeln!(
            file,
            "            Currency::{} => vec![{}],",
            &currency.alpha3, &country_list
        )
        .unwrap();
    }
    writeln!(file, "        }};").unwrap();
    writeln!(file, "        territories.sort();").unwrap();
    writeln!(file, "        territories").unwrap();
    writeln!(file, "    }}").unwrap();
    writeln!(file).unwrap();

    writeln!(file, "    /// Returns the currency's symbol").unwrap();
    writeln!(file, "    ///").unwrap();
    writeln!(
        file,
        "    /// This method will return the symbol commonly used to represent the"
    )
    .unwrap();
    writeln!(
        file,
        "    /// currency. In case there is no symbol associated the international"
    )
    .unwrap();
    writeln!(file, "    /// currency symbol will be returned.").unwrap();
    writeln!(file, "    ///").unwrap();
    writeln!(file, "    /// # Example").unwrap();
    writeln!(file, "    ///").unwrap();
    writeln!(file, "    /// ```").unwrap();
    writeln!(file, "    /// use iso_currency::Currency;").unwrap();
    writeln!(file, "    ///").unwrap();
    writeln!(
        file,
        "    /// assert_eq!(format!(\"{{}}\", Currency::EUR.symbol()), \"€\");"
    )
    .unwrap();
    writeln!(
        file,
        "    /// assert_eq!(format!(\"{{}}\", Currency::XXX.symbol()), \"¤\");"
    )
    .unwrap();
    writeln!(file, "    /// ```").unwrap();
    writeln!(file, "    pub fn symbol(self) -> CurrencySymbol {{").unwrap();
    writeln!(file, "        match self {{").unwrap();
    for currency in data.iter() {
        let subunit_symbol = match &currency.subunit_symbol {
            Some(v) => format!("Some(\"{}\")", &v),
            None => "None".into(),
        };
        writeln!(
            file,
            "            Currency::{} => CurrencySymbol::new(\"{}\", {}),",
            &currency.alpha3, &currency.symbol, subunit_symbol
        )
        .unwrap();
    }
    writeln!(file, "        }}").unwrap();
    writeln!(file, "    }}").unwrap();
    writeln!(file).unwrap();

    writeln!(
        file,
        "    /// Create a currency instance from a ISO 4217 character code"
    )
    .unwrap();
    writeln!(file, "    ///").unwrap();
    writeln!(file, "    /// # Example").unwrap();
    writeln!(file, "    ///").unwrap();
    writeln!(file, "    /// ```").unwrap();
    writeln!(file, "    /// use iso_currency::Currency;").unwrap();
    writeln!(file, "    ///").unwrap();
    writeln!(
        file,
        "    /// assert_eq!(Currency::from_code(\"EUR\"), Some(Currency::EUR));"
    )
    .unwrap();
    writeln!(file, "    /// ```").unwrap();
    writeln!(
        file,
        "    pub fn from_code(code: &str) -> Option<Currency> {{"
    )
    .unwrap();
    writeln!(file, "        if code.len() != 3 {{").unwrap();
    writeln!(file, "            return None;").unwrap();
    writeln!(file, "        }}").unwrap();
    writeln!(file, "        match code {{").unwrap();
    for currency in data.iter() {
        writeln!(
            file,
            "            \"{}\" => Some(Currency::{}),",
            &currency.alpha3, &currency.alpha3
        )
        .unwrap();
    }
    writeln!(file, "            _ => None,").unwrap();
    writeln!(file, "        }}").unwrap();
    writeln!(file, "    }}").unwrap();
    writeln!(file).unwrap();

    writeln!(
        file,
        "    /// Create a currency instance from a ISO 4217 numeric code"
    )
    .unwrap();
    writeln!(file, "    ///").unwrap();
    writeln!(file, "    /// # Example").unwrap();
    writeln!(file, "    ///").unwrap();
    writeln!(file, "    /// ```").unwrap();
    writeln!(file, "    /// use iso_currency::Currency;").unwrap();
    writeln!(file, "    ///").unwrap();
    writeln!(
        file,
        "    /// assert_eq!(Currency::from_numeric(978), Some(Currency::EUR));"
    )
    .unwrap();
    writeln!(file, "    /// ```").unwrap();
    writeln!(
        file,
        "    pub fn from_numeric(numeric_code: u16) -> Option<Currency> {{"
    )
    .unwrap();
    writeln!(file, "        match numeric_code {{").unwrap();
    for currency in data.iter() {
        writeln!(
            file,
            "            {} => Some(Currency::{}),",
            currency.numeric, &currency.alpha3
        )
        .unwrap();
    }
    writeln!(file, "            _ => None,").unwrap();
    writeln!(file, "        }}").unwrap();
    writeln!(file, "    }}").unwrap();

    writeln!(
        file,
        "    /// Returns the exponent of a currency (number of decimal places)"
    )
    .unwrap();
    writeln!(
        file,
        "    /// For example, 1.00 Euro a 2 subunits so this will return Some(2) for EUR."
    )
    .unwrap();
    writeln!(file, "    ///").unwrap();
    writeln!(
        file,
        "    /// This returns an optional value because some currencies don't have a subunit."
    )
    .unwrap();
    writeln!(file, "    ///").unwrap();
    writeln!(file, "    /// # Example").unwrap();
    writeln!(file, "    ///").unwrap();
    writeln!(file, "    /// ```").unwrap();
    writeln!(file, "    /// use iso_currency::Currency;").unwrap();
    writeln!(file, "    ///").unwrap();
    writeln!(
        file,
        "    /// assert_eq!(Currency::EUR.exponent(), Some(2));"
    )
    .unwrap();
    writeln!(
        file,
        "    /// assert_eq!(Currency::JPY.exponent(), Some(0));"
    )
    .unwrap();
    writeln!(file, "    /// ```").unwrap();
    writeln!(file, "    pub fn exponent(self) -> Option<u16> {{").unwrap();
    writeln!(file, "        match self {{").unwrap();
    for currency in data.iter() {
        let value = match &currency.exponent {
            Some(v) => format!("Some({})", v),
            None => "None".into(),
        };
        writeln!(
            file,
            "            Currency::{} => {},",
            &currency.alpha3, &value
        )
        .unwrap();
    }
    writeln!(file, "        }}").unwrap();
    writeln!(file, "    }}").unwrap();
    writeln!(file).unwrap();

    writeln!(
        file,
        "    /// Returns how many of the subunits equal the main unit of the currency"
    )
    .unwrap();
    writeln!(file, "    /// For example there are a 100 cents in 1 Euro so this will return Some(100) for EUR.").unwrap();
    writeln!(file, "    ///").unwrap();
    writeln!(
        file,
        "    /// This returns an optional value because some currencies don't have a subunit."
    )
    .unwrap();
    writeln!(file, "    ///").unwrap();
    writeln!(file, "    /// # Example").unwrap();
    writeln!(file, "    ///").unwrap();
    writeln!(file, "    /// ```").unwrap();
    writeln!(file, "    /// use iso_currency::Currency;").unwrap();
    writeln!(file, "    ///").unwrap();
    writeln!(
        file,
        "    /// assert_eq!(Currency::EUR.subunit_fraction(), Some(100));"
    )
    .unwrap();
    writeln!(file, "    /// ```").unwrap();
    writeln!(file, "    pub fn subunit_fraction(self) -> Option<u16> {{").unwrap();
    writeln!(file, "        match self {{").unwrap();
    for currency in data.iter() {
        let value = match &currency.exponent {
            Some(v) => format!("Some(10_u16.pow({}))", v),
            None => "None".into(),
        };
        writeln!(
            file,
            "            Currency::{} => {},",
            &currency.alpha3, &value
        )
        .unwrap();
    }
    writeln!(file, "        }}").unwrap();
    writeln!(file, "    }}").unwrap();
    writeln!(file).unwrap();

    writeln!(file, "pub fn is_fund(self) -> bool {{").unwrap();
    writeln!(file, "    match self {{").unwrap();
    for currency in data.iter() {
        writeln!(
            file,
            "        Currency::{} => {},",
            &currency.alpha3, currency.is_fund
        )
        .unwrap();
    }
    writeln!(file, "    }}").unwrap();
    writeln!(file, "}}").unwrap();
    
    writeln!(file, "pub fn is_special(self) -> bool {{").unwrap();
    writeln!(file, "    match self {{").unwrap();
    for currency in data.iter() {
        writeln!(
            file,
            "        Currency::{} => {},",
            &currency.alpha3, currency.is_special
        )
        .unwrap();
    }
    writeln!(file, "    }}").unwrap();
    writeln!(file, "}}").unwrap();

    writeln!(file, "pub fn is_superseded(self) -> Option<Self> {{").unwrap();
    writeln!(file, "    match self {{").unwrap();
    for currency in data.iter() {
        writeln!(
            file,
            "        Currency::{} => {},",
            &currency.alpha3, match currency.is_superseded {
                Some(ref v) => format!("Some(Currency::{})", v),
                None => "None".into(),
            }
        )
        .unwrap();
    }
    writeln!(file, "    }}").unwrap();
    writeln!(file, "}}").unwrap();

    writeln!(file, "pub fn latest(self) -> Self {{").unwrap();
    writeln!(file, "    match self {{").unwrap();
    for currency in data.iter() {
        writeln!(
            file,
            "        Currency::{} => Currency::{},",
            &currency.alpha3, match currency.is_superseded {
                Some(ref v) => v.clone(),
                None => currency.alpha3.clone(),
            }
        )
        .unwrap();
    }
    writeln!(file, "    }}").unwrap();
    writeln!(file, "}}").unwrap();

    writeln!(file, "pub fn flags(self) -> Vec<Flag> {{").unwrap();
    writeln!(file, "    match self {{").unwrap();
    for currency in data.iter() {
        writeln!(
            file,
            "        Currency::{} => vec![{}],",
            &currency.alpha3, flags_vec(currency)
        )
        .unwrap();
    }
    writeln!(file, "    }}").unwrap();
    writeln!(file, "}}").unwrap();
    
    writeln!(file, "pub fn has_flag(self, flag: Flag) -> bool {{").unwrap();
    writeln!(file, "    match self {{").unwrap();
    for currency in data.iter() {
        writeln!(
            file,
            "        Currency::{} => Currency::{}.flags().contains(&flag),",
            &currency.alpha3, &currency.alpha3,
        )
        .unwrap();
    }
    writeln!(file, "    }}").unwrap();
    writeln!(file, "}}").unwrap();

    writeln!(file, "}}").unwrap();
    writeln!(file).unwrap();
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
