use std::process::Command;
use std::fs;
use std::path::Path;
use std::fs::File;
use std::io::{BufRead, BufReader};
use float_cmp::approx_eq;


/// Reads the `time` and `novelty` columns from a CSV file.
fn load_csv(path: &str) -> Vec<(f32, f32)> {
    let file = File::open(path).expect("Failed to open file");
    let reader = BufReader::new(file);

    reader
        .lines()
        .skip(1) // Skip header
        .map(|line| {
            let line = line.expect("Failed to read line");
            let parts: Vec<&str> = line.trim().split(',').collect();
            assert!(
                parts.len() == 2,
                "Expected two columns in each row, got: {:?}",
                parts
            );
            let time = parts[0].parse::<f32>().expect("Failed to parse time");
            let novelty = parts[1].parse::<f32>().expect("Failed to parse novelty");
            (time, novelty)
        })
        .collect()
}


// tests a computed CSV file against a reference CSV file
#[test]
fn test_novelty_output_against_reference() {
    // Paths
    let test_audio = "assets/LJ037-0171.wav";
    let generated_csv = "LJ037-0171.csv";
    let reference_csv = "reference/LJ037-0171.csv";

    // Clean old output if it exists
    if Path::new(generated_csv).exists() {
        fs::remove_file(generated_csv).unwrap();
    }

    // Call your binary with args (builds and runs main.rs)
    let status = Command::new(env!("CARGO_BIN_EXE_novelty_rust"))
        .args([
            test_audio,
            generated_csv,
            "--window-length", "2048",
            "--hop-length", "128",
            "--gamma", "10.0",
            "--norm",
        ])
        .status()
        .expect("Failed to execute program");

    assert!(status.success());

    let ref_data = load_csv(reference_csv);
    let act_data = load_csv(generated_csv);

    assert_eq!(
        ref_data.len(),
        act_data.len(),
        "CSV files have different number of rows"
    );

    let tol = 1e-3;

    for (i, ((t_ref, n_ref), (t_act, n_act))) in ref_data.iter().zip(act_data.iter()).enumerate() {
        assert!(
            approx_eq!(f32, *t_ref, *t_act, epsilon = tol),
            "Time mismatch at index {}: expected {}, got {}",
            i,
            t_ref,
            t_act
        );
        assert!(
            approx_eq!(f32, *n_ref, *n_act, epsilon = tol),
            "Novelty mismatch at index {}: expected {}, got {}",
            i,
            n_ref,
            n_act
        );
    }
}