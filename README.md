# biblib

[![Crates.io](https://img.shields.io/crates/v/biblib.svg)](https://crates.io/crates/biblib)
[![Documentation](https://docs.rs/biblib/badge.svg)](https://docs.rs/biblib)

A comprehensive Rust library for parsing, managing, and deduplicating academic citations. `biblib` provides robust support for multiple citation formats, intelligent deduplication, and extensive metadata handling.

> **Note:** This crate is under active development. There may be some breaking changes between versions. Please check the changelog before upgrading.

## Features

### Multiple Format Support

- **RIS (Research Information Systems)**
  - Full tag support
  - Author name parsing
  - Journal abbreviations
- **PubMed/MEDLINE**

  - Complete field coverage
  - MeSH terms support
  - Affiliation handling

- **EndNote XML**

  - Full XML schema support
  - Unicode handling
  - Custom field mapping

- **CSV with Custom Mappings**
  - Configurable headers
  - Multiple delimiters
  - Flexible field mapping

### Intelligent Deduplication

- DOI-based matching
- Smart title comparison using Jaro-Winkler distance
- Journal name/abbreviation matching
- Configurable matching thresholds
- Year-based grouping for performance
- Parallel processing support

### Rich Metadata Support

- Complete author information with affiliations
- Journal details (name, abbreviation, ISSN)
- DOIs and other identifiers (PMID, PMC ID)
- Comprehensive citation metadata

### Size Optimization

This crate can be minimized like so:

```shell
cargo add biblib --no-default-features --features lite
```

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
biblib = "0.2.0"  # All features enabled by default
```

Or select specific features:

```toml
[dependencies]
biblib = { version = "0.2.0", default-features = false, features = ["csv", "ris"] }
```

Available features:

- `csv` - CSV format support
- `pubmed` - PubMed/MEDLINE format support
- `xml` - EndNote XML support (requires quick-xml)
- `ris` - RIS format support
- `dedupe` - Citation deduplication (requires rayon and strsim)

All features are enabled by default. Disable `default-features` to select specific ones.

## Quick Start

### Basic Citation Parsing

```rust
use biblib::{CitationParser, RisParser};

// Parse RIS format
let input = r#"TY  - JOUR
TI  - Example Article
AU  - Smith, John
ER  -"#;

let parser = RisParser::new();
let citations = parser.parse(input).unwrap();
println!("Title: {}", citations[0].title);
```

### Citation Deduplication

```rust
use biblib::dedupe::{Deduplicator, DeduplicatorConfig};

// Configure deduplication
let config = DeduplicatorConfig {
    group_by_year: true,
    run_in_parallel: true,
};

let deduplicator = Deduplicator::with_config(config);
let duplicate_groups = deduplicator.find_duplicates(&citations).unwrap();

for group in duplicate_groups {
    println!("Original: {}", group.unique.title);
    for duplicate in group.duplicates {
        println!("  Duplicate: {}", duplicate.title);
    }
}
```

### CSV with Custom Mappings

```rust
use biblib::csv::{CsvParser, CsvConfig};

let mut config = CsvConfig::new();
config.set_header_mapping("title", vec!["Article Name".to_string()])
     .set_delimiter(b';');

let parser = CsvParser::with_config(config);
let citations = parser.parse("Article Name;Author;Year\nExample Paper;Smith J;2023").unwrap();
```

## Supported Fields

| Field      | Description                          | RIS | PubMed | EndNote XML | CSV |
| ---------- | ------------------------------------ | --- | ------ | ----------- | --- |
| Title      | Work title                           | ✓   | ✓      | ✓           | ✓   |
| Authors    | Author names and affiliations        | ✓   | ✓      | ✓           | ✓   |
| Journal    | Journal name and abbreviation        | ✓   | ✓      | ✓           | ✓   |
| Year       | Publication year                     | ✓   | ✓      | ✓           | ✓   |
| Volume     | Journal volume                       | ✓   | ✓      | ✓           | ✓   |
| Issue      | Journal issue                        | ✓   | ✓      | ✓           | ✓   |
| Pages      | Page range                           | ✓   | ✓      | ✓           | ✓   |
| DOI        | Digital Object Identifier            | ✓   | ✓      | ✓           | ✓   |
| PMID       | PubMed ID                            | ✓   | ✓      | -           | ✓   |
| PMC ID     | PubMed Central ID                    | ✓   | ✓      | ✓           | ✓   |
| Abstract   | Abstract text                        | ✓   | ✓      | ✓           | ✓   |
| Keywords   | Keywords/tags                        | ✓   | ✓      | ✓           | ✓   |
| Language   | Publication language                 | ✓   | ✓      | ✓           | ✓   |
| Publisher  | Publisher information                | ✓   | -      | ✓           | ✓   |
| URLs       | Related URLs                         | ✓   | -      | ✓           | ✓   |
| ISSN       | International Standard Serial Number | ✓   | ✓      | ✓           | ✓   |
| MeSH Terms | Medical Subject Headings             | -   | ✓      | -           | -   |

## Advanced Usage

### Customizing Deduplication

```rust
use biblib::dedupe::{Deduplicator, DeduplicatorConfig};

// Fine-tune deduplication settings
let config = DeduplicatorConfig {
    group_by_year: true,     // Enable year-based grouping
    run_in_parallel: true,   // Enable parallel processing
};

let deduplicator = Deduplicator::with_config(config);
```

### Error Handling

```rust
use biblib::{CitationParser, RisParser, CitationError};

let result = RisParser::new().parse("invalid input");
match result {
    Ok(citations) => println!("Parsed {} citations", citations.len()),
    Err(CitationError::InvalidFormat(msg)) => eprintln!("Parse error: {}", msg),
    Err(e) => eprintln!("Other error: {}", e),
}
```

## Performance Considerations

- Use year-based grouping for large datasets (> 1000 citations)
- Enable parallel processing for better deduplication performance
- Consider using CSV format for very large datasets
- Pre-process and normalize data when possible

## Thread Safety

All parser implementations are thread-safe and can be shared between threads. The deduplication engine supports parallel processing through the `run_in_parallel` configuration option.

## Contributing

We welcome contributions! Please feel free to submit pull requests. For major changes, please open an issue first to discuss what you would like to change.

Make sure to update tests as appropriate and follow the existing code style.

## License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Project Status

This crate is under active. There may be some breaking changes.

## Support

- Documentation: [docs.rs](https://docs.rs/biblib)
- Issues: [GitHub Issues](https://github.com/AliAzlanDev/biblib/issues)
- Discussions: [GitHub Discussions](https://github.com/AliAzlanDev/biblib/discussions)
