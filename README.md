# C++20 Modules Tools

`cpp20modules_tools` is a toolkit for handling C++20 modules, containing two main functionalities:
1. `agg-ddi`: Aggregate module dependency information files (`.ddi`) and module information files (`.CXXModules.json`) into a single module information file (`.CXXModules.json`).
2. `gen-modmap`: Generate `.modmap` file for a specified compiler.

## Directory Structure

```plaintext
cpp20modules_tools/
├── Cargo.toml
└── src/
    ├── bin/
    │   ├── agg-ddi.rs
    │   └── gen-modmap.rs
    ├── lib.rs
```

## Dependencies

This project relies on the following Rust libraries:
- `clap`: For command-line argument parsing.
- `serde` and `serde_json`: For JSON serialization and deserialization.

## Installation

1. Clone this repository:
   ```sh
   git clone https://github.com/your_username/cpp20modules_tools.git
   cd cpp20modules_tools
   ```

2. Build the project using `cargo`:
   ```sh
   cargo build --release
   ```

## Usage

### agg-ddi

The `agg-ddi` tool aggregates module dependency information files (`.ddi`) and module information files (`.CXXModules.json`) into a single module information file (`.CXXModules.json`).

#### Command-line Arguments

- `-m, --cpp20modules-info <cpp20modules-info>`: Paths to the input `.CXXModules.json` files. Multiple input files can be specified using multiple `-m` options.
- `-d, --ddi <DDI>`: Paths to the input `.ddi` files. Multiple input files can be specified using multiple `-d` options.
- `-o, --output <OUTPUT>`: Path to the output `.CXXModules.json` file.

#### Example

```sh
cargo run --bin agg-ddi -- -d input1.ddi -d input2.ddi -m input1.CXXModules.json -m input2.CXXModules.json -o output.CXXModules.json
```

### gen-modmap

The `gen-modmap` tool generates a `.modmap` file for a specified compiler.

#### Command-line Arguments

- `-c, --compiler <COMPILER>`: Compiler type (currently supports Clang, GCC, and MSVC).
- `-m, --cpp20modules-info <cpp20modules-info>`: Path to the `.CXXModules.json` file.
- `-d, --ddi <DDI>`: Path to the `.ddi` JSON file.
- `-o, --output <OUTPUT>`: Path to the output `.modmap` file.

#### Example

```sh
cargo run --bin gen-modmap -- --compiler clang --cpp20modules-info input.CXXModules.json --ddi ddi.json --output output.modmap
```

## Running Tests

Run all unit tests using the following command:

```sh
cargo test
```

## License

This project is licensed under the [Apache-2.0 License](LICENSE).

## Contributing

Contributions are welcome! Please submit a Pull Request or open an Issue.
