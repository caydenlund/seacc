use seacc::source_reader::SourceReader;
use std::io::Cursor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = concat!(
        "??=include <stdio.h>\n",
        "int main() ??<\n",
        "    printf(\"Hello??/n\");\n",
        "    return 0;\n",
        "??>\n",
    );

    println!("Raw source:\n");
    for ch in source.chars() {
        print!("{ch}");
    }

    println!("\n\nWith trigraph replacement:\n");
    let cursor = Cursor::new(source.as_bytes().to_vec());
    let reader = SourceReader::new("<demo>", cursor);

    for result in reader {
        let spanned = result.expect("Failed to read character");
        print!("{}", spanned.value);
    }

    Ok(())
}
