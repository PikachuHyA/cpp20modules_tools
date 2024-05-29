use clap::Parser;
use cpp20modules_tools::{Compiler, Cpp20ModulesInfo, Ddi, read_file, write_file};
use std::error::Error;

/// Command line arguments struct
#[derive(Parser, Debug)]
#[command(version, about = "Generate .modmap file for the specified compiler")]
struct Args {
    /// Compiler (Clang, GCC, MSVC)
    #[arg(short = 'c', long)]
    compiler: String,

    /// Path to the cpp20modules_info JSON file
    #[arg(short = 'm', long)]
    cpp20modules_info: String,

    /// Path to the ddi JSON file
    #[arg(short, long)]
    ddi: String,

    /// Path to the output file (usually ends with .modmap)
    #[arg(short, long)]
    output: String,
}

/// Generate the .modmap content based on the provided data
fn generate_modmap(
    compiler: Compiler,
    cpp20modules_info: &Cpp20ModulesInfo,
    ddi: &Ddi,
) -> Result<String, Box<dyn Error>> {
    let mut modmap_content = String::new();

    for rule in &ddi.rules {
        if rule.provides.is_empty() {
            continue;
        }

        let provide = &rule.provides[0];
        let logical_name = &provide.logical_name;

        if let Some(module) = cpp20modules_info.modules.get(logical_name) {
            // Generate module mapping for the specified compiler
            match compiler {
                Compiler::Clang | Compiler::Gcc => {
                    modmap_content.push_str(&format!("-x c++-module\n"));
                    modmap_content.push_str(&format!("-fmodule-output={}\n", module.bmi));
                }
                Compiler::Msvc => {
                    modmap_content.push_str(&format!("/module:output {}\n", module.bmi));
                }
            }

            // Add requires mapping
            for require in &rule.requires {
                if let Some(reference) = cpp20modules_info.references.get(&require.logical_name) {
                    match compiler {
                        Compiler::Clang | Compiler::Gcc => {
                            modmap_content.push_str(&format!(
                                "-fmodule-file={}={}\n",
                                require.logical_name, reference.path
                            ));
                        }
                        Compiler::Msvc => {
                            modmap_content.push_str(&format!(
                                "/module:reference {}={}\n",
                                require.logical_name, reference.path
                            ));
                        }
                    }
                } else {
                    return Err(format!(
                        "Reference for required module '{}' not found",
                        require.logical_name
                    )
                        .into());
                }
            }

            // Only handle the first rule for now (as in provided example)
            break;
        }
    }

    Ok(modmap_content)
}

fn main() -> Result<(), Box<dyn Error>> {
    // Parse command line arguments
    let args = Args::parse();

    // Parse the compiler
    let compiler = Compiler::from_str(&args.compiler).map_err(|e| {
        eprintln!("{}", e);
        e
    })?;

    // Read and parse the cpp20modules_info JSON file
    let cpp20modules_info: Cpp20ModulesInfo = read_file(&args.cpp20modules_info).map_err(|e| {
        eprintln!(
            "Failed to parse cpp20modules_info file '{}': {}",
            args.cpp20modules_info, e
        );
        e
    })?;

    // Read and parse the ddi JSON file
    let ddi: Ddi = read_file(&args.ddi).map_err(|e| {
        eprintln!("Failed to parse ddi file '{}': {}", args.ddi, e);
        e
    })?;

    // Generate the .modmap content
    let modmap_content = generate_modmap(compiler, &cpp20modules_info, &ddi)?;

    // Write the .modmap content to the output file
    write_file(&args.output, &modmap_content).map_err(|e| {
        eprintln!("Failed to write to output file '{}': {}", args.output, e);
        e
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use cpp20modules_tools::{Module, Reference, Provide, Require, Rule};

    fn get_sample_cpp20modules_info() -> Cpp20ModulesInfo {
        Cpp20ModulesInfo {
            modules: {
                let mut modules = HashMap::new();
                modules.insert(
                    "bar".to_string(),
                    Module {
                        bmi: "cmake-build-debug/CMakeFiles/bar.dir/bar.pcm".into(),
                        is_private: false,
                    },
                );
                modules
            },
            references: {
                let mut references = HashMap::new();
                references.insert(
                    "bar".to_string(),
                    Reference {
                        lookup_method: "by-name".into(),
                        path: "CMakeFiles/bar.dir/bar.pcm".into(),
                    },
                );
                references.insert(
                    "foo".to_string(),
                    Reference {
                        lookup_method: "by-name".into(),
                        path: "CMakeFiles/foo.dir/foo.pcm".into(),
                    },
                );
                references
            },
            usages: HashMap::new(),
        }
    }

    fn get_sample_ddi() -> Ddi {
        Ddi {
            revision: 0,
            rules: vec![Rule {
                primary_output: "CMakeFiles/bar.dir/bar.cpp.o".into(),
                provides: vec![Provide {
                    is_interface: true,
                    logical_name: "bar".into(),
                    source_path: "demo/bar.cpp".into(),
                }],
                requires: vec![Require {
                    logical_name: "foo".into(),
                }],
            }],
            version: 1,
        }
    }

    #[test]
    fn test_generate_modmap_clang() {
        let compiler = Compiler::Clang;
        let cpp20modules_info = get_sample_cpp20modules_info();
        let ddi = get_sample_ddi();

        let result = generate_modmap(compiler, &cpp20modules_info, &ddi).unwrap();
        let expected = "-x c++-module\n-fmodule-output=cmake-build-debug/CMakeFiles/bar.dir/bar.pcm\n-fmodule-file=foo=CMakeFiles/foo.dir/foo.pcm\n";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_generate_modmap_gcc() {
        let compiler = Compiler::Gcc;
        let cpp20modules_info = get_sample_cpp20modules_info();
        let ddi = get_sample_ddi();

        let result = generate_modmap(compiler, &cpp20modules_info, &ddi).unwrap();
        let expected = "-x c++-module\n-fmodule-output=cmake-build-debug/CMakeFiles/bar.dir/bar.pcm\n-fmodule-file=foo=CMakeFiles/foo.dir/foo.pcm\n";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_generate_modmap_msvc() {
        let compiler = Compiler::Msvc;
        let cpp20modules_info = get_sample_cpp20modules_info();
        let ddi = get_sample_ddi();

        let result = generate_modmap(compiler, &cpp20modules_info, &ddi).unwrap();
        let expected = "/module:output cmake-build-debug/CMakeFiles/bar.dir/bar.pcm\n/module:reference foo=CMakeFiles/foo.dir/foo.pcm\n";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_generate_modmap_missing_reference() {
        let compiler = Compiler::Clang;
        let mut cpp20modules_info = get_sample_cpp20modules_info();
        cpp20modules_info.references.remove("foo");
        let ddi = get_sample_ddi();

        let result = generate_modmap(compiler, &cpp20modules_info, &ddi);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_write_file() {
        let test_content = "{\"key\": \"value\"}";
        let path = "test.json";

        write_file(path, test_content).unwrap();
        let result: HashMap<String, String> = read_file(path).unwrap();

        assert_eq!(result.get("key"), Some(&"value".into()));

        std::fs::remove_file(path).unwrap();
    }
}
