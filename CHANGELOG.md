# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2025-06-10

### Added

- **New `Date` struct**: Comprehensive date support with year, month, and day fields
- **Enhanced date parsing**: Support for complex date formats across all parsers:
  - PubMed: "2020 Jun 9", "2023 May 30", "2023 Jan 3", "2023"
  - RIS: "1999/12/25/Christmas edition", "2023/05/30", "2023", "2023//"
  - EndNote XML: Support for year, month, day attributes in XML elements
  - CSV: Year-only parsing with fallback support

### Changed

- **BREAKING**: Citations now use `date: Date` field instead of `year: Option<i32>`
- **BREAKING**: Deduplicator now uses `date.year` for year-based grouping
- All parsers updated to populate the new `date` field with complete date information
- Backward compatibility maintained through deprecated `year` field

### Deprecated

- `Citation.year` field is now deprecated in favor of `Citation.date.year`
- Will be removed in version 0.4.0

### Migration Guide

Replace `citation.year` with `citation.date.year` in your code:

```rust
// Old
if let Some(year) = citation.year {
    println!("Published in {}", year);
}

// New
if let Some(year) = citation.date.year {
    println!("Published in {}", year);
    if let Some(month) = citation.date.month {
        println!("Month: {}", month);
    }
    if let Some(day) = citation.date.day {
        println!("Day: {}", day);
    }
}
```

## [0.2.3] - 2025-06-09

### Fixed

- Improved modularity of CSV and XML feature. Fixes compilation errors when default features are not in use

### Changed

- License changed to MIT or Apache-2.0

## [0.2.2] - 2025-01-31

### Fixed

- Fixed RIS parser line to handle tags like T1, A2, etc.

## [0.2.1] - 2025-01-28

### Added

- New `detect_and_parse` function for automatic format detection and parsing
- Support for automatic detection of RIS, PubMed, and EndNote XML formats

## [0.2.0] - 2025-01-28

### Added

- New `source` field in `Citation` struct to track citation origin
- `.with_source()` method on all parsers (RIS, PubMed, EndNote XML, CSV) to specify citation source
- `source_preferences` option in `DeduplicatorConfig` for controlling unique citation selection
- Cargo features for optional components:
  - `csv` - Enable CSV format support
  - `pubmed` - Enable PubMed/MEDLINE format support
  - `xml` - Enable EndNote XML support
  - `ris` - Enable RIS format support
  - `dedupe` - Enable citation deduplication
  - All features enabled by default

### Changed

- Enhanced unique citation selection logic in deduplicator:
  1. Prefers citations from sources listed in `source_preferences`
  2. Falls back to citations with abstracts if no source preference matches
  3. Prefers citations with DOIs if abstracts exist in both citations
  4. Uses first citation as fallback if all above criteria are equal

## [0.1.0] - 2025-01-25

### Added

- Initial release with core functionality
- Support for multiple citation formats (RIS, PubMed, EndNote XML, CSV)
- Citation deduplication engine
- Comprehensive metadata handling
