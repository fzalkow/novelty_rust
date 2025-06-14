# Energy-Based Novelty Function in Rust

This project computes an **energy-based novelty function** from a mono WAV audio file using Rust.
It is inspired by the [FMP Notebooks](https://www.audiolabs-erlangen.de/resources/MIR/FMP/C6/C6S1_NoveltyEnergy.html), and primarily serves as a learning exercise in idiomatic Rust for audio processing.

---

## ✨ What Is a Novelty Function?

A **novelty function** highlights changes in an audio signal, such as onsets or dynamic shifts.
This implementation uses short-time energy with optional logarithmic compression and half-wave rectification.

---

## 🛠️ Build Instructions

Ensure you have [Rust](https://www.rust-lang.org/tools/install) installed. To compile the project in release mode:

```bash
cargo build --release
```

---

## 🚀 Running the Program

To run the program:

```bash
cargo run --release -- <INPUT_WAV> <OUTPUT_CSV> [--window-length <u32>] [--hop-length <u32>] [--gamma <f32>] [--norm <bool>]
```

### Example:

```bash
cargo run --release -- assets/LJ037-0171.wav LJ037-0171.csv --gamma 10.0 --norm
```

* `--window-length`: Window size for energy computation (default: 1024)
* `--hop-length`: Hop size between frames (default: 256)
* `--gamma`: Logarithmic compression parameter (default: 10.0)
* `--norm`: Normalize the output between 0–1 (default: true)

> ⚠️ Input must be a mono WAV file.

---

## 🧪 Testing

Run integration and unit tests with:

```bash
cargo test
```

---

## 🧹 Linting

Check code quality and suggestions with:

```bash
cargo clippy
```

---

## 📁 Output

The program outputs a CSV file with two columns:

```text
time,novelty
0.00000,0.00000
0.01161,0.12457
...
```

This can be visualized using Python/Matplotlib, Excel, or similar tools.

---

## 📚 References

* [FMP Notebooks: Energy-Based Novelty](https://www.audiolabs-erlangen.de/resources/MIR/FMP/C6/C6S1_NoveltyEnergy.html)

