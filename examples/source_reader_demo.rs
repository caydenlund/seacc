use seacc::source_reader::SourceReader;
use std::{
    fs::File,
    io::{BufReader, Read, Write},
};
use tempfile::NamedTempFile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = NamedTempFile::new()?;
    writeln!(file, "??=include <stdio.h>")?;
    writeln!(file, "int main() ??<")?;
    writeln!(file, "    printf(\"Hello??/n\");")?;
    writeln!(file, "    return 0;")?;
    writeln!(file, "??>")?;
    file.flush()?;

    let path_owned = file.path().to_str().unwrap().to_string();
    println!("Reading file: '{path_owned}'\n");

    let mut s = String::new();
    BufReader::new(File::open(&path_owned)?).read_to_string(&mut s)?;
    for ch in s.chars() {
        print!("{ch}");
    }

    println!("\n\nWith trigraph replacement:\n");
    let reader = SourceReader::new(&path_owned).expect("Failed to create reader");

    for result in reader {
        let spanned = result.expect("Failed to read character");
        print!("{}", spanned.value);
    }

    Ok(())
}
