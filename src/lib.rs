use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};

// Define the structure for the Cpp20ModulesInfo (input JSON data)
#[derive(Debug, Deserialize, Serialize)]
pub struct Module {
    pub bmi: String,
    #[serde(rename = "is-private")]
    pub is_private: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Reference {
    #[serde(rename = "lookup-method")]
    pub lookup_method: String,
    pub path: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Cpp20ModulesInfo {
    pub modules: HashMap<String, Module>,
    pub references: HashMap<String, Reference>,
    pub usages: HashMap<String, Vec<String>>,
}

// Define the structure for the Ddi (input JSON data)
#[derive(Debug, Deserialize, Serialize)]
pub struct Require {
    #[serde(rename = "logical-name")]
    pub logical_name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Provide {
    #[serde(rename = "is-interface")]
    pub is_interface: bool,
    #[serde(rename = "logical-name")]
    pub logical_name: String,
    #[serde(rename = "source-path")]
    pub source_path: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Rule {
    #[serde(rename = "primary-output")]
    pub primary_output: String,
    #[serde(default)]
    pub provides: Vec<Provide>,
    #[serde(default)]
    pub requires: Vec<Require>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Ddi {
    pub revision: i32,
    pub rules: Vec<Rule>,
    pub version: i32,
}

// Supported compilers
#[derive(Debug, PartialEq)]
pub enum Compiler {
    Clang,
    Gcc,
    Msvc,
}

impl Compiler {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "clang" => Ok(Compiler::Clang),
            "gcc" => Ok(Compiler::Gcc),
            "msvc" => Ok(Compiler::Msvc),
            _ => Err(format!("Unsupported compiler: {}", s)),
        }
    }
}

pub fn read_file<T: for<'de> Deserialize<'de>>(path: &str) -> Result<T, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let data = serde_json::from_reader(reader)?;
    Ok(data)
}

pub fn write_file(path: &str, content: &str) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    writer.write_all(content.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_write_file() {
        let test_content = r#"
        {
            "revision": 1,
            "rules": [],
            "version": 1
        }
        "#;
        let path = "test_ddi.json";

        // write
        write_file(path, test_content).unwrap();

        // read
        let read_ddi: Ddi = read_file(path).unwrap();
        assert_eq!(read_ddi.revision, 1);
        assert!(read_ddi.rules.is_empty());
        assert_eq!(read_ddi.version, 1);

        // clean up
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_compiler_from_str() {
        assert_eq!(Compiler::from_str("clang").unwrap(), Compiler::Clang);
        assert_eq!(Compiler::from_str("gcc").unwrap(), Compiler::Gcc);
        assert_eq!(Compiler::from_str("msvc").unwrap(), Compiler::Msvc);
        assert!(Compiler::from_str("unknown").is_err());
    }
}