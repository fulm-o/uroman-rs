use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

fn main() {
    let path = Path::new("src/decompositions.rs");
    let mut file = BufWriter::new(File::create(path).unwrap());

    writeln!(
        &mut file,
        "pub static DECOMPOSITIONS: phf::Map<char, (&'static str, &'static str)> = phf::phf_map! {{"
    ).unwrap();

    let input = BufReader::new(File::open("data/UnicodeData.txt").unwrap());

    for line in input.lines() {
        let line = line.unwrap();
        let fields: Vec<&str> = line.split(';').collect();
        if fields.len() > 5 && !fields[5].is_empty() {
            let codepoint_hex = fields[0];
            let codepoint =
                char::from_u32(u32::from_str_radix(codepoint_hex, 16).unwrap()).unwrap();

            let decomp_field = fields[5];
            let (tag, decomp_codes) = if decomp_field.starts_with('<') {
                let mut parts = decomp_field.splitn(2, ' ');
                (parts.next().unwrap_or(""), parts.next().unwrap_or(""))
            } else {
                ("", decomp_field)
            };

            let mut decomp_str = String::new();
            for code_hex in decomp_codes.split(' ') {
                if !code_hex.is_empty()
                    && let Ok(code) = u32::from_str_radix(code_hex, 16)
                    && let Some(ch) = char::from_u32(code)
                {
                    decomp_str.push(ch);
                }
            }

            writeln!(
                &mut file,
                "    '\\u{{{:x}}}' => (\"{}\", \"{}\"),",
                codepoint as u32,
                tag,
                decomp_str.escape_default()
            ).unwrap();
        }
    }

    writeln!(&mut file, "}};").unwrap();
}
