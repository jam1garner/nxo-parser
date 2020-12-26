# nxo-parser

Rust parsers for Nintendo Switch executable formats

```rust
use nxo_parser::NsoFile;
use binread::BinReaderExt;

let mut reader: impl Read + Seek = /* ... */;

let nso = reader.read_le().unwrap();

println!("Is .text compressed? {:?}", nso.flags.text_compressed());

let decompressed_code = nso.get_text(&mut reader).unwrap();
```
