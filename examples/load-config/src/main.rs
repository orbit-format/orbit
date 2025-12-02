use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use orbit_core::{self, serializer::to_json_string_pretty};

fn main() -> Result<()> {
    let config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("sample.orbit");
    let source = fs::read_to_string(&config_path)
        .with_context(|| format!("failed to read {}", config_path.display()))?;
    println!("Loaded config from {}", config_path.display());

    let ast = orbit_core::parse(&source).context("failed to parse Orbit document")?;
    println!("AST:\n{:#?}", ast);

    let value = orbit_core::evaluate_ast(&ast).context("failed to evaluate Orbit document")?;
    let json = to_json_string_pretty(&value).context("failed to serialize value to JSON")?;
    println!("\nResolved value as JSON:\n{}", json);

    Ok(())
}
