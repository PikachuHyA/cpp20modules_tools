use clap::Parser;
use cpp20modules_tools::{Cpp20ModulesInfo, Ddi, Module, Reference, Provide, Rule, read_file, write_file};
use std::collections::HashMap;

/// Command line arguments struct
#[derive(Parser, Debug)]
#[command(version, about = "Aggregate module dependency information files (.ddi) and module information files (Cpp20ModulesInfo.json) into a .CXXModules.json file.")]
struct Args {
    /// Path to the cpp20modules_info JSON files
    #[arg(short = 'm', long)]
    cpp20modules_info: Vec<String>,

    /// Path to the ddi JSON files
    #[arg(short, long)]
    ddi: Vec<String>,

    /// Path to the output file (usually ends with .CXXModules.json)
    #[arg(short, long)]
    output: String,
}

/// Function to transform Ddi to Cpp20ModulesInfo
fn transform_ddi_to_cpp20modules_info(ddi: &Ddi) -> Cpp20ModulesInfo {
    let mut cpp20modules_info = Cpp20ModulesInfo {
        modules: HashMap::new(),
        references: HashMap::new(),
        usages: HashMap::new(),
    };

    for rule in &ddi.rules {
        for provide in &rule.provides {
            let logical_name = provide.logical_name.clone();

            cpp20modules_info.modules.insert(
                logical_name.clone(),
                Module {
                    bmi: rule.primary_output.clone(),
                    is_private: !provide.is_interface,
                },
            );

            cpp20modules_info.references.insert(
                logical_name,
                Reference {
                    lookup_method: "by-name".to_string(),
                    path: rule.primary_output.clone(),
                },
            );
        }
    }

    cpp20modules_info.usages = HashMap::new();
    cpp20modules_info
}

/// Function to merge two Cpp20ModulesInfo objects
fn merge_cpp20modules_info(
    info1: &mut Cpp20ModulesInfo,
    info2: Cpp20ModulesInfo,
) {
    info1.modules.extend(info2.modules);
    info1.references.extend(info2.references);
    // Handle other fields as needed (e.g., info1.usages)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args = Args::parse();

    let mut cpp20modules_info = Cpp20ModulesInfo {
        modules: HashMap::new(),
        references: HashMap::new(),
        usages: HashMap::new(),
    };

    // Process each ddi input file
    for input_path in &args.ddi {
        let ddi: Ddi = read_file(input_path).map_err(|e| {
            eprintln!("Failed to parse input file '{}': {}", input_path, e);
            e
        })?;
        let transformed_info = transform_ddi_to_cpp20modules_info(&ddi);
        merge_cpp20modules_info(&mut cpp20modules_info, transformed_info);
    }

    // Process each cpp20modules_info input file
    for input_path in &args.cpp20modules_info {
        let info: Cpp20ModulesInfo = read_file(input_path).map_err(|e| {
            eprintln!("Failed to parse input file '{}': {}", input_path, e);
            e
        })?;
        merge_cpp20modules_info(&mut cpp20modules_info, info);
    }

    // Convert the Cpp20ModulesInfo struct back to JSON
    let final_json = serde_json::to_string_pretty(&cpp20modules_info).map_err(|e| {
        eprintln!("Failed to serialize final JSON: {}", e);
        e
    })?;

    // Write the final JSON to the output file
    write_file(&args.output, &final_json).map_err(|e| {
        eprintln!("Failed to write to output file '{}': {}", args.output, e);
        e
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_ddi_to_cpp20modules_info() {
        let provide = Provide {
            is_interface: true,
            logical_name: "example".to_string(),
            source_path: "src/example.cppm".to_string(),
        };
        let rule = Rule {
            primary_output: "example.o".to_string(),
            provides: vec![provide],
            requires: vec![],
        };
        let ddi = Ddi {
            revision: 1,
            rules: vec![rule],
            version: 1,
        };

        let cpp20modules_info = transform_ddi_to_cpp20modules_info(&ddi);

        assert!(cpp20modules_info.modules.contains_key("example"));
        let module = cpp20modules_info.modules.get("example").unwrap();
        assert_eq!(module.bmi, "example.o");
        assert!(!module.is_private);

        assert!(cpp20modules_info.references.contains_key("example"));
        let reference = cpp20modules_info.references.get("example").unwrap();
        assert_eq!(reference.lookup_method, "by-name");
        assert_eq!(reference.path, "example.o");

        assert!(cpp20modules_info.usages.is_empty());
    }

    #[test]
    fn test_transform_ddi_to_cpp20modules_info_multiple_rules() {
        let provide1 = Provide {
            is_interface: true,
            logical_name: "example1".to_string(),
            source_path: "src/example1.cppm".to_string(),
        };
        let rule1 = Rule {
            primary_output: "example1.o".to_string(),
            provides: vec![provide1],
            requires: vec![],
        };

        let provide2 = Provide {
            is_interface: false,
            logical_name: "example2".to_string(),
            source_path: "src/example2.cppm".to_string(),
        };
        let rule2 = Rule {
            primary_output: "example2.o".to_string(),
            provides: vec![provide2],
            requires: vec![],
        };

        let ddi = Ddi {
            revision: 1,
            rules: vec![rule1, rule2],
            version: 1,
        };

        let cpp20modules_info = transform_ddi_to_cpp20modules_info(&ddi);

        assert!(cpp20modules_info.modules.contains_key("example1"));
        let module1 = cpp20modules_info.modules.get("example1").unwrap();
        assert_eq!(module1.bmi, "example1.o");
        assert!(!module1.is_private);

        assert!(cpp20modules_info.modules.contains_key("example2"));
        let module2 = cpp20modules_info.modules.get("example2").unwrap();
        assert_eq!(module2.bmi, "example2.o");
        assert!(module2.is_private);

        assert!(cpp20modules_info.references.contains_key("example1"));
        let reference1 = cpp20modules_info.references.get("example1").unwrap();
        assert_eq!(reference1.lookup_method, "by-name");
        assert_eq!(reference1.path, "example1.o");

        assert!(cpp20modules_info.references.contains_key("example2"));
        let reference2 = cpp20modules_info.references.get("example2").unwrap();
        assert_eq!(reference2.lookup_method, "by-name");
        assert_eq!(reference2.path, "example2.o");

        assert!(cpp20modules_info.usages.is_empty());
    }

    #[test]
    fn test_transform_ddi_to_cpp20modules_info_empty_rules() {
        let ddi = Ddi {
            revision: 1,
            rules: vec![],
            version: 1,
        };

        let cpp20modules_info = transform_ddi_to_cpp20modules_info(&ddi);

        assert!(cpp20modules_info.modules.is_empty());
        assert!(cpp20modules_info.references.is_empty());
        assert!(cpp20modules_info.usages.is_empty());
    }
}
