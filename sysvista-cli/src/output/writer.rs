use std::fs;
use std::io;
use std::path::Path;

use super::schema::SysVistaOutput;

pub fn write_json(output: &SysVistaOutput, path: &Path) -> io::Result<()> {
    let json = serde_json::to_string_pretty(output)?;
    fs::write(path, json)?;
    Ok(())
}
