use assert_cmd::prelude::*;
use std::fs::{self, File};
use std::process::Command;

fn assert_uroman_output(input_path: &str, expected_output_path: &str) {
    let expected_output = fs::read_to_string(expected_output_path).unwrap_or_else(|_| {
        panic!("could not read expected output file: {expected_output_path}")
    });

    let expected_output_normalized = expected_output.replace("\r\n", "\n");

    let mut cmd = Command::cargo_bin("uroman-rs").unwrap();
    let input = File::open(input_path).unwrap();
    let output = cmd
        .stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let actual_output_normalized = String::from_utf8(output).unwrap().replace("\r\n", "\n");

    assert_eq!(actual_output_normalized, expected_output_normalized);
}

#[test]
fn test_multi_script_romanization() {
    assert_uroman_output(
        "tests/test/multi-script.txt",
        "tests/test/multi-script.uroman-ref.txt",
    );
}
