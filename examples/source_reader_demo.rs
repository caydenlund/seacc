use seacc::source_reader::SourceReader;
use std::{
    fs::File,
    io::{BufReader, Read, Write},
};
use tempfile::NamedTempFile;

fn main() -> std::io::Result<()> {
    let mut file = NamedTempFile::new()?;
    writeln!(file, "??=include <stdio.h>")?;
    writeln!(file, "int main() ??<")?;
    writeln!(file, "    printf(\"Hello??/n\");")?;
    writeln!(file, "    return 0;")?;
    writeln!(file, "??>")?;
    file.flush()?;

    let path = file.path().to_str().unwrap();
    println!("Reading file: '{path}'\n");

    let mut s = String::new();
    BufReader::new(File::open(path)?).read_to_string(&mut s)?;
    for ch in s.chars() {
        print!("{ch}");
    }

    println!("\n\nWith trigraph replacement:\n");
    let reader = SourceReader::new(path)?;

    for result in reader {
        let spanned = result?;
        print!("{}", spanned.value);
    }

    Ok(())
}
