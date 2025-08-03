//! Main library for the uroman-rs project.
//!
//! This library provides the `Uroman` struct, which is the main entry point
//! for romanizing strings. It loads romanization rules from data files and
//! applies them to input text.

#![allow(clippy::too_many_arguments)]

use clap::ValueEnum;
use indexmap::IndexMap;
use regex::Regex;
use serde::Serialize;
use serde_json::Value as JsonValue;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::io::{self, BufRead, Write};
use std::sync::LazyLock;
use thiserror::Error;
use unicode_normalization::UnicodeNormalization;
use unicode_properties::UnicodeGeneralCategory;

pub use crate::edge::Edge;
use crate::lattice::Lattice;
use crate::utils::slot_value_in_double_colon_del_list;

mod decompositions;
mod edge;
mod lattice;
mod rom_rule;
mod utils;

use rom_rule::{RomRule, RomRules};

static KAYAH_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"kayah\s+(\S+)\s*$").unwrap());
static MENDE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"m\d+\s+(\S+)\s*$").unwrap());
static SPACE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\S\s+\S").unwrap());

#[derive(ValueEnum, Clone, Debug, Default)]
pub enum RomFormat {
    #[default]
    Str,
    Edges,
    ALTS,
    Lattice,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq, PartialOrd)]
#[serde(untagged)]
pub enum RomanizationResult {
    Str(String),
    Edges(Vec<Edge>),
}

impl RomanizationResult {
    pub fn to_output_string(&self) -> Result<String, RomanizationError> {
        match self {
            RomanizationResult::Str(s) => Ok(s.clone()),
            RomanizationResult::Edges(edges) => Ok(serde_json::to_string_pretty(edges)?),
        }
    }
}

#[derive(Error, Debug)]
pub enum RomanizationError {
    #[error("Failed to serialize the result to JSON: {0}")]
    SerializationFailed(#[from] serde_json::Error),

    #[error(transparent)]
    Io(#[from] io::Error),

    #[error("Internal logic error: {0}")]
    InternalError(String),
}

/// Represents a value that can be an integer, float, or string.
#[derive(Debug, Clone)]
enum Value {
    Int(i64),
    Float(f64),
    String(String),
}

/// Represents a script with its properties.
#[allow(unused)]
#[derive(Debug, Clone)]
struct Script {
    pub script_name: String,
    pub direction: Option<String>,
    pub abugida_default_vowels: Vec<String>,
    pub alt_script_names: Vec<String>,
    pub languages: Vec<String>,
    pub abugida_regexes: Option<(Regex, Regex)>,
}

// #[derive(Default, Debug)]
// struct NumPropDefaults {
//     pub value: Option<f64>,
//     pub num_base: Option<i64>,
//     pub is_large_power: Option<bool>,
// }

#[derive(Debug, Clone)]
struct AbugidaCacheEntry {
    base_rom: Option<String>,
    base_rom_plus_vowel: Option<String>,
    modified_rom: String,
}

/// The main struct for romanization.
///
/// It holds the romanization rules and provides methods to romanize strings.
/// This corresponds to the `Uroman` class in the Python implementation.
#[derive(Debug, Default, Clone)]
pub struct Uroman {
    rom_rules: RomRules,
    scripts: HashMap<String, Script>,
    dict_bool: HashMap<(String, String), bool>,
    dict_str: HashMap<(String, String), String>,
    num_props: HashMap<char, HashMap<String, Value>>,
    percentage_markers: HashSet<String>,
    fraction_connectors: HashSet<String>,
    plus_signs: HashSet<String>,
    minus_signs: HashSet<String>,
    hangul_rom: RefCell<HashMap<char, String>>,
    abugida_cache: RefCell<HashMap<(String, String), AbugidaCacheEntry>>,
}

impl Uroman {
    pub fn new() -> Self {
        let mut uroman = Self {
            rom_rules: IndexMap::with_capacity(42979),
            scripts: HashMap::with_capacity(179),
            dict_bool: HashMap::with_capacity(44366),
            dict_str: HashMap::with_capacity(122770),
            num_props: HashMap::with_capacity(1599),
            percentage_markers: HashSet::new(),
            fraction_connectors: HashSet::new(),
            minus_signs: HashSet::new(),
            plus_signs: HashSet::new(),
            hangul_rom: HashMap::new().into(),
            abugida_cache: HashMap::new().into(),
        };
        uroman.load_resource_files();
        uroman
    }

    /// Registers all prefixes of a string `s` for efficient lookup later.
    pub fn register_s_prefix(&mut self, s: &str) {
        let mut prefix = String::with_capacity(s.chars().count());
        for c in s.chars() {
            prefix.push(c);
            self.dict_bool
                .insert(("s-prefix".to_string(), prefix.clone()), true);
        }
    }

    // /// Retrieves the numerical properties for a given character.
    // ///
    // /// This method looks up the character in the `num_props` map.
    // /// If the character is not found, it returns default values.
    // fn get_num_props(&self, c: char) -> NumPropDefaults {
    //     self.num_props
    //         .get(&c)
    //         .map_or_else(NumPropDefaults::default, |props| {
    //             let value = props.get("value").and_then(|v| match v {
    //                 Value::Int(i) => Some(*i as f64),
    //                 Value::Float(f) => Some(*f),
    //                 _ => None,
    //             });
    //             let num_base = props.get("base").and_then(|v| match v {
    //                 Value::Int(i) => Some(*i),
    //                 _ => None,
    //             });
    //             let is_large_power = props.get("is-large-power").and_then(|v| match v {
    //                 Value::Int(1) => Some(true),
    //                 _ => None,
    //             });

    //             NumPropDefaults {
    //                 value,
    //                 num_base,
    //                 is_large_power,
    //             }
    //         })
    // }

    fn load_resource_files(&mut self) {
        self.load_rom_file(
            include_str!("../data/romanization-auto-table.txt"),
            "ud",
            "rom",
        );
        self.load_rom_file(
            include_str!("../data/UnicodeDataOverwrite.txt"),
            "ow",
            "u2r",
        );
        self.load_rom_file(
            include_str!("../data/romanization-table.txt"),
            "man",
            "rom",
        );
        self.load_chinese_pinyin_file(include_str!("../data/Chinese_to_Pinyin.txt"));
        self.load_script_file(include_str!("../data/Scripts.txt"));
        self.load_unicode_data_props(include_str!("../data/UnicodeDataProps.txt"));
        self.load_unicode_data_props(include_str!("../data/UnicodeDataPropsCJK.txt"));
        self.load_unicode_data_props(include_str!("../data/UnicodeDataPropsHangul.txt"));
        self.load_num_props(include_str!("../data/NumProps.jsonl"));
        self.add_thai_cancellation_rules();
    }

    /// Loads numerical properties from a JSONL file (e.g., NumProps.jsonl).
    fn load_num_props(&mut self, file: &'static str) {
        for line in file.lines() {
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }

            let json: JsonValue = serde_json::from_str(line).expect("invalid JSON map");
            if let Some(obj) = json.as_object()
                && let Some(txt_val) = obj.get("txt")
                && let Some(txt) = txt_val.as_str()
                && let Some(key_char) = txt.chars().next()
            {
                let mut num_prop_map = HashMap::new();
                for (key, val) in obj {
                    match val {
                        JsonValue::Number(n) => {
                            if n.is_i64() {
                                num_prop_map.insert(key.clone(), Value::Int(n.as_i64().unwrap()));
                            } else if n.is_f64() {
                                num_prop_map.insert(key.clone(), Value::Float(n.as_f64().unwrap()));
                            }
                        }
                        JsonValue::String(s) => {
                            num_prop_map.insert(key.clone(), Value::String(s.clone()));
                        }
                        JsonValue::Bool(b) => {
                            num_prop_map.insert(key.clone(), Value::Int(if *b { 1 } else { 0 }));
                        }
                        _ => {}
                    }
                }
                self.num_props.insert(key_char, num_prop_map);
            }
        }
    }

    /// Loads Unicode data properties from a file (e.g., UnicodeDataProps.txt).
    fn load_unicode_data_props(&mut self, file: &'static str) {
        for line in file.lines() {
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }

            if let Some(script_name) =
                utils::slot_value_in_double_colon_del_list(line, "script-name")
            {
                if let Some(chars_str) = utils::slot_value_in_double_colon_del_list(line, "char") {
                    for c in chars_str.chars() {
                        self.dict_str.insert(
                            ("script".to_string(), c.to_string()),
                            script_name.to_string(),
                        );
                    }
                }
                if let Some(vowel_sign_str) =
                    utils::slot_value_in_double_colon_del_list(line, "vowel-sign")
                {
                    for c in vowel_sign_str.chars() {
                        self.dict_bool
                            .insert(("is-vowel-sign".to_string(), c.to_string()), true);
                    }
                }
                if let Some(medial_consonant_sign_str) =
                    utils::slot_value_in_double_colon_del_list(line, "medial-consonant-sign")
                {
                    for c in medial_consonant_sign_str.chars() {
                        self.dict_bool.insert(
                            ("is-medial-consonant-sign".to_string(), c.to_string()),
                            true,
                        );
                    }
                }
                if let Some(virama_str) =
                    utils::slot_value_in_double_colon_del_list(line, "sign-virama")
                {
                    for c in virama_str.chars() {
                        self.dict_bool
                            .insert(("is-virama".to_string(), c.to_string()), true);
                    }
                }
            }
        }
    }

    /// Loads a script definition file (e.g., Scripts.txt).
    fn load_script_file(&mut self, file: &'static str) {
        for line in file.lines() {
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }

            if let Some(script_name) =
                utils::slot_value_in_double_colon_del_list(line, "script-name")
            {
                let lc_script_name = script_name.to_lowercase();
                if self.scripts.contains_key(&lc_script_name) {
                    // Handle duplicate script names (Python version warns and ignores)
                    continue;
                }

                let direction = utils::slot_value_in_double_colon_del_list(line, "direction")
                    .map(|s| s.to_string());
                let abugida_default_vowel_s =
                    utils::slot_value_in_double_colon_del_list(line, "abugida-default-vowel")
                        .unwrap_or("");
                let abugida_default_vowels = if abugida_default_vowel_s.is_empty() {
                    vec![]
                } else {
                    abugida_default_vowel_s
                        .split([',', ';'])
                        .map(|s| s.trim().to_string())
                        .collect()
                };
                let alt_script_name_s =
                    utils::slot_value_in_double_colon_del_list(line, "alt-script-name")
                        .unwrap_or("");
                let alt_script_names = if alt_script_name_s.is_empty() {
                    vec![]
                } else {
                    alt_script_name_s
                        .split([',', ';'])
                        .map(|s| s.trim().to_string())
                        .collect()
                };
                let language_s =
                    utils::slot_value_in_double_colon_del_list(line, "language").unwrap_or("");
                let languages = if language_s.is_empty() {
                    vec![]
                } else {
                    language_s
                        .split([',', ';'])
                        .map(|s| s.trim().to_string())
                        .collect()
                };

                let abugida_regexes = if !abugida_default_vowels.is_empty() {
                    let vowels_regex1 = abugida_default_vowels.join("|");
                    let vowels_regex2 = abugida_default_vowels
                        .iter()
                        .map(|v| format!("{}+", v))
                        .collect::<Vec<_>>()
                        .join("|");

                    let re1 =
                        Regex::new(&format!(r"([cfghkmnqrstxy]?y)({})-?$", vowels_regex2)).unwrap();
                    let re2 = Regex::new(&format!(
                        r"([bcdfghjklmnpqrstvwxyz]+)({})-?$",
                        vowels_regex1
                    ))
                    .unwrap();

                    Some((re1, re2))
                } else {
                    None
                };

                let new_script = Script {
                    script_name: script_name.to_string(),
                    direction,
                    abugida_default_vowels,
                    alt_script_names: alt_script_names.clone(),
                    languages: languages.clone(),
                    abugida_regexes,
                };

                self.scripts.insert(lc_script_name, new_script.clone());

                for alt_script_name in alt_script_names {
                    self.scripts
                        .insert(alt_script_name.to_lowercase(), new_script.clone());
                }
            }
        }
    }

    fn load_rom_file(&mut self, file: &'static str, provenance: &str, file_format: &str) {
        for line in file.lines() {
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }

            if file_format == "u2r" {
                let u_str = match slot_value_in_double_colon_del_list(line, "u") {
                    Some(s) => s,
                    None => continue,
                };

                let s = match u32::from_str_radix(u_str, 16)
                    .ok()
                    .and_then(std::char::from_u32)
                {
                    Some(c) => c,
                    None => continue,
                };

                if let Some(tone_mark) = slot_value_in_double_colon_del_list(line, "tone-mark") {
                    self.dict_str.insert(
                        ("tone-mark".to_string(), s.to_string()),
                        tone_mark.to_string(),
                    );
                }

                if let Some(syllable_info) =
                    slot_value_in_double_colon_del_list(line, "syllable-info")
                {
                    self.dict_str.insert(
                        ("syllable-info".to_string(), s.to_string()),
                        syllable_info.to_string(),
                    );
                }

                if let Some(syllable_info) = slot_value_in_double_colon_del_list(line, "pic") {
                    self.dict_str.insert(
                        ("pic".to_string(), s.to_string()),
                        syllable_info.to_string(),
                    );
                }

                if let Some(syllable_info) = slot_value_in_double_colon_del_list(line, "name") {
                    self.dict_str.insert(
                        ("name".to_string(), s.to_string()),
                        syllable_info.to_string(),
                    );
                }

                // 'u2r'フォーマットの行からもRomRuleを生成する
                // この部分は、sとtを抽出してから既存のfrom_lineロジックに渡すか、
                // RomRule生成に必要な部分をここに再実装する。
                // ここでは簡易的にRomRuleを直接構築する。
                if let Some(rule) = RomRule::from_line(line, provenance, file_format, self) {
                    self.add_rom_rule(rule);
                }
            } else if let Some(rule) = RomRule::from_line(line, provenance, file_format, self) {
                self.add_rom_rule(rule);
            }
        }
    }

    fn add_rom_rule(&mut self, rule: RomRule) {
        if rule.is_minus_sign {
            self.minus_signs.insert(rule.s.clone());
        }
        if rule.is_plus_sign {
            self.plus_signs.insert(rule.s.clone());
        }
        if rule.fraction_connector {
            self.fraction_connectors.insert(rule.s.clone());
        }

        if rule.is_large_power {
            self.dict_bool
                .insert(("is-large-power".to_string(), rule.s.clone()), true);
        }

        self.register_s_prefix(&rule.s);

        let old_rules = self.rom_rules.entry(rule.s.clone()).or_default();

        // Python: `and not (lcodes or ...)`
        let is_unconditional = rule.is_unconditional();

        let should_overwrite = old_rules.len() == 1 && {
            let old_rule = &old_rules[0];
            (old_rule.prov == "ud" || old_rule.prov == "ow") && is_unconditional
        };

        // println!(
        // "LOAD: s='{}', prov='{}', is_uncond={}, should_ow={}",
        //     rule.s, rule.prov, is_unconditional, should_overwrite
        // );

        if should_overwrite {
            *old_rules = vec![rule];
        } else {
            old_rules.push(rule);
        }
    }

    /// Loads and processes the Chinese to Pinyin mapping file.
    fn load_chinese_pinyin_file(&mut self, file: &'static str) {
        for line in file.lines() {
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }

            if let Some((chinese, pinyin_with_accent)) = line.split_once(char::is_whitespace) {
                // `de_accent_pinyin` logic: NFD decomposition to separate base chars and accents.
                let rom: String = pinyin_with_accent
                    .nfd()
                    .filter(|c| {
                        !matches!(
                            c.general_category_group(),
                            unicode_properties::GeneralCategoryGroup::Mark
                        )
                    })
                    .collect::<String>()
                    .replace('ü', "u");

                let rule = RomRule::new_simple(chinese.to_string(), &rom, "rom pinyin");
                self.rom_rules
                    .entry(chinese.to_string())
                    .or_default()
                    .push(rule);
                self.register_s_prefix(chinese);
            }
        }
    }

    /// A helper to get a string value from `dict_str`, returning `""` if not found.
    pub fn dict_str_get(&self, k1: &str, k2_char: char) -> &str {
        self.dict_str
            .get(&(k1.to_string(), k2_char.to_string()))
            .map(|s| s.as_str()) // Option<&String> -> Option<&str>
            .unwrap_or("") // None -> ""
    }

    /// A helper to get a boolean value from `dict_bool`, returning `false` if not found.
    /// This mimics the behavior of Python's `defaultdict(bool)`.
    pub fn dict_bool_get(&self, k1: &str, k2: &str) -> bool {
        self.dict_bool
            .get(&(k1.to_string(), k2.to_string()))
            .copied()
            .unwrap_or(false)
    }

    pub fn second_rom_filter(&self, c: &str, rom: Option<&str>) -> Option<String> {
        if c.is_empty() {
            return rom.map(|s| s.to_string());
        }

        let rom_str = match rom {
            Some(r) if r.contains(' ') => r,
            _ => return rom.map(|s| s.to_string()),
        };

        let name = self.chr_name(c.chars().next().unwrap());

        if name.contains("MYANMAR VOWEL SIGN KAYAH")
            && let Some(cap) = KAYAH_RE.captures(rom_str)
        {
            return Some(cap.get(1).unwrap().as_str().to_string());
        }
        if name.contains("MENDE KIKAKUI SYLLABLE")
            && let Some(cap) = MENDE_RE.captures(rom_str)
        {
            return Some(cap.get(1).unwrap().as_str().to_string());
        }
        if SPACE_RE.is_match(rom_str) {
            return Some(c.to_string());
        }

        rom.map(|s| s.to_string())
    }

    /// Gets the numeric value of a character from the loaded `num_props` data.
    /// This is the correct replacement for Python's `unicodedata.numeric()`.
    ///
    /// It looks up the character, then the "value" key, and converts the result to `f64`.
    pub fn get_numeric_value(&self, c: char) -> Option<f64> {
        self.num_props
            .get(&c)
            .and_then(|props| props.get("value"))
            .and_then(|val| match val {
                Value::Int(i) => Some(*i as f64),
                Value::Float(f) => Some(*f),
                _ => None,
            })
    }

    /// Checks if a character is a non-spacing mark.
    pub fn char_is_nonspacing_mark(&self, c: char) -> bool {
        use unicode_properties::UnicodeGeneralCategory;
        matches!(
            c.general_category(),
            unicode_properties::GeneralCategory::NonspacingMark
        )
    }

    /// Checks if a character is a format control character.
    pub fn char_is_format_char(&self, c: char) -> bool {
        use unicode_properties::UnicodeGeneralCategory;
        matches!(
            c.general_category(),
            unicode_properties::GeneralCategory::Format
        )
    }

    pub fn chr_name(&self, c: char) -> String {
        // Check for an overridden name in dict_str.
        if let Some(name) = self.dict_str.get(&("name".to_string(), c.to_string())) {
            return name.clone();
        }
        unicode_names2::name(c)
            .map(|n| n.to_string())
            .unwrap_or_default()
    }

    /// Converts a Korean Hangul character to its Latin alphabet representation.
    ///
    /// This is a special algorithmic romanization that decomposes a Hangul syllable
    /// into its constituent Jamo (lead, vowel, tail) and maps them to roman characters.
    /// The results are cached for performance.
    pub fn unicode_hangul_romanization(&self, c: char) -> Option<String> {
        if let Some(cached_rom) = self.hangul_rom.borrow().get(&c) {
            return Some(cached_rom.clone());
        }

        let cp = c as u32;

        // Check if the codepoint is within the Hangul Syllables range (AC00–D7A3).
        if (0xAC00..=0xD7A3).contains(&cp) {
            let code = cp - 0xAC00;

            // Calculate the indices for the lead, vowel, and tail (Jamo).
            let lead_index = (code / (28 * 21)) as usize;
            let vowel_index = ((code / 28) % 21) as usize;
            let tail_index = (code % 28) as usize;

            let rom = format!(
                "{}{}{}",
                HANGUL_LEADS[lead_index], HANGUL_VOWELS[vowel_index], HANGUL_TAILS[tail_index]
            );

            // Remove the placeholder hyphen '-'.
            let rom = rom.replace('-', "");

            self.hangul_rom.borrow_mut().insert(c, rom.clone());

            Some(rom)
        } else {
            None
        }
    }

    pub fn unicode_hangul_romanization_str(&mut self, s: &str, pass_through_p: bool) -> String {
        let mut result = String::new();
        for c in s.chars() {
            if let Some(rom) = self.unicode_hangul_romanization(c) {
                result.push_str(&rom);
            } else if pass_through_p {
                result.push(c);
            }
        }
        result
    }

    /// Returns the script name of a character.
    ///
    /// This is derived from `UnicodeDataProps*.txt` and stored in `dict_str`.
    /// Returns an empty string if not found.
    pub fn chr_script_name(&self, c: char) -> String {
        self.dict_str
            .get(&("script".to_string(), c.to_string()))
            .cloned()
            .unwrap_or_default()
    }

    /// Adds automatic cancellation rules for the Thai script.
    ///
    /// This method programmatically generates rules to handle the Thai character
    /// THANTHAKHAT (`\u0E4C`), which indicates that the preceding character(s)
    /// should not be pronounced (and thus not romanized).
    fn add_thai_cancellation_rules(&mut self) {
        let thai_cancellation_mark = '\u{0E4C}';
        for cp in 0x0E01..0x0E4C {
            if let Some(c) = std::char::from_u32(cp) {
                let s = format!("{}{}", c, thai_cancellation_mark);

                let rules_for_s = self.rom_rules.entry(s.clone()).or_default();
                if rules_for_s.is_empty() {
                    let rule = RomRule::new_simple(s.clone(), "", "auto cancel letter");
                    rules_for_s.push(rule);
                    self.register_s_prefix(&s);
                }
            }
        }

        let thai_consonants = (0x0E01..0x0E2F).filter_map(std::char::from_u32);

        let thai_vowel_modifiers = ['\u{0E31}', '\u{0E47}']
            .into_iter()
            .chain((0x0E33..=0x0E3B).filter_map(std::char::from_u32));

        for c1 in thai_consonants.clone() {
            for v in thai_vowel_modifiers.clone() {
                let s = format!("{}{}{}", c1, v, thai_cancellation_mark);

                let rules_for_s = self.rom_rules.entry(s.clone()).or_default();
                if rules_for_s.is_empty() {
                    let rule = RomRule::new_simple(s.clone(), "", "auto cancel syllable");
                    rules_for_s.push(rule);
                    self.register_s_prefix(&s);
                }
            }
        }
    }

    /// Romanizes a given string.
    pub fn romanize_string(
        &self,
        s: &str,
        lcode: Option<&str>,
        rom_format: Option<&RomFormat>,
    ) -> Result<RomanizationResult, RomanizationError> {
        let rom_format = rom_format.unwrap_or(&RomFormat::Str);
        let mut lat = Lattice::new(s, self, lcode);

        lat.pick_tibetan_vowel_edge();
        lat.prep_braille();
        lat.add_romanization();
        lat.add_numbers();
        lat.add_braille_numbers();
        lat.add_rom_fall_back_singles();

        match rom_format {
            RomFormat::Str => {
                let best_edges = lat.best_rom_edge_path(0, s.chars().count(), false);

                Ok(
                    RomanizationResult::Str(
                        best_edges.iter().map(|edge| edge.txt()).collect::<String>(),
                    )
                )
            },
            RomFormat::Edges => {
                let best_edges = lat.best_rom_edge_path(0, s.chars().count(), false);

                Ok(RomanizationResult::Edges(best_edges))
            },
            RomFormat::ALTS => {
                let mut best_edges = lat.best_rom_edge_path(0, s.chars().count(), false);

                lat.add_alternatives(&mut best_edges);
                Ok(RomanizationResult::Edges(best_edges))
            },
            RomFormat::Lattice => {
                let mut all_edges = lat.all_edges(0, s.chars().count());
                lat.add_alternatives(&mut all_edges);
                Ok(RomanizationResult::Edges(all_edges))
            }
        }
    }

    /// Romanizes a stream of text line by line and writes the output to another stream.
    ///
    /// This method efficiently processes large amounts of text by reading from a buffered
    /// reader and writing to a writer without loading the entire content into memory.
    ///
    /// # Arguments
    ///
    /// * `reader` - A buffered reader for the input stream (e.g., a file or stdin).
    /// * `writer` - A writer for the output stream (e.g., a file or stdout).
    /// * `lcode` - An optional ISO 639-3 language code to specify the script.
    ///
    /// # Errors
    ///
    /// This function will return an `io::Error` if any I/O operation fails during
    /// reading from the `reader` or writing to the `writer`.
    pub fn romanize_file<R: BufRead, W: Write>(
        &self,
        mut reader: R,
        mut writer: W,
        lcode: Option<&str>,
        rom_format: &RomFormat,
        max_lines: Option<usize>,
        silent: bool,
    ) -> Result<(), RomanizationError> {
        let mut line_number = 0;
        let mut non_utf8_chars_total = 0;
        let mut n_error_messages_output = 0;
        let max_n_error_messages = 10;

        let mut buffer = vec![];
        let default_lcode = lcode;
        let lcode_directive = "::lcode ";

        while reader.read_until(b'\n', &mut buffer)? > 0 {
            line_number += 1;

            let original_len = buffer.len();
            let line_str = String::from_utf8_lossy(&buffer);
            let replaced_len = line_str.len();
            if replaced_len < original_len {
                non_utf8_chars_total += 1;
                if n_error_messages_output < max_n_error_messages {
                    eprintln!(
                        "Detected encoding error on line {}: non-UTF-8 characters were replaced.",
                        line_number
                    );
                    n_error_messages_output += 1;
                } else if n_error_messages_output == max_n_error_messages {
                    eprintln!("Too many encoding errors. No further errors reported.");
                    n_error_messages_output += 1;
                }
            }
            let mut line_trimmed = &*line_str;

            if line_trimmed.ends_with('\n') {
                line_trimmed = &line_trimmed[..line_trimmed.len() - 1];
            }
            if line_trimmed.ends_with('\r') {
                line_trimmed = &line_trimmed[..line_trimmed.len() - 1];
            }

            if let Some(rest_of_line) = line_trimmed.strip_prefix(lcode_directive) {
                let parts: Vec<&str> = rest_of_line.splitn(2, char::is_whitespace).collect();
                let (lcode, text_to_romanize) =
                    (parts.first().cloned(), parts.get(1).cloned().unwrap_or(""));

                let result = self.romanize_string(text_to_romanize, lcode, Some(rom_format));

                match rom_format {
                    RomFormat::Str => {
                        let prefix = format!("{}{}{} ", lcode_directive, lcode.unwrap_or(""), "");
                        let output = prefix + &result?.to_output_string().unwrap();
                        writeln!(writer, "{}", output)?;
                    }
                    _ => {
                        let meta_edge = format!(r#"[0,0,"","lcode: {}"]"#, lcode.unwrap_or(""));
                        let result_json = result?.to_output_string().unwrap();
                        if let Some(stripped) = result_json.strip_prefix('[') {
                            writeln!(writer, "[{},{}", meta_edge, stripped)?;
                        } else {
                            writeln!(writer, "{}", result_json)?;
                        }
                    }
                }
            } else {
                let result = self.romanize_string(line_trimmed, default_lcode, Some(rom_format));
                let output = result?
                    .to_output_string()
                    .expect("JSON serialization failed");
                writeln!(writer, "{}", output)?;
            }

            if !silent {
                if line_number % 1000 == 0 {
                    eprint!("{}", line_number);
                } else if line_number % 100 == 0 {
                    eprint!(".");
                }
                if line_number % 100 == 0 {
                    io::stderr().flush()?;
                }
            }

            if let Some(max) = max_lines
                && line_number >= max
            {
                break;
            }
            buffer.clear();
        }

        if !silent && line_number > 0 {
            eprintln!();
        }
        if non_utf8_chars_total > 0 {
            eprintln!(
                "Total number of lines with non-UTF-8 characters: {}",
                non_utf8_chars_total
            );
        }

        writer.flush()?;
        Ok(())
    }
}

static HANGUL_LEADS: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    "g gg n d dd r m b bb s ss - j jj c k t p h"
        .split_whitespace()
        .collect()
});
static HANGUL_VOWELS: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    "a ae ya yae eo e yeo ye o wa wai oe yo u weo we wi yu eu yi i"
        .split_whitespace()
        .collect()
});
static HANGUL_TAILS: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    "- g gg gs n nj nh d l lg lm lb ls lt lp lh m b bs s ss ng j c k t p h"
        .split_whitespace()
        .collect()
});
