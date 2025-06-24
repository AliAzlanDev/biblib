# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### BREAKING CHANGES

- **Removed source tracking functionality**: The `source` field has been completely removed from the `Citation` struct and all parser implementations
- **Removed `with_source()` methods**: All parsers no longer have the `with_source()` method
- **Updated `detect_and_parse()` function**: Now takes only one parameter (`content`) instead of two (`content`, `source`)
- **Updated deduplication API**: Added `find_duplicates_with_sources()` method for source-aware deduplication

### Migration Guide

If you were using source tracking in your application, you'll need to handle source tracking at the application level:

**Before (v0.2.x):**

```rust
let parser = RisParser::new().with_source("PubMed");
let citations = parser.parse(input).unwrap();
let source = citations[0].source.clone(); // "PubMed"
```

**After (v0.3.x):**

```rust
let parser = RisParser::new();
let citations = parser.parse(input).unwrap();
// Handle source tracking in your application:
let source = "PubMed"; // manage this in your app
```

**For `detect_and_parse()`:**

```rust
// Before
let (citations, format) = detect_and_parse(content, "PubMed").unwrap();

// After
let (citations, format) = detect_and_parse(content).unwrap();
// Track source separately in your application
```

**For deduplication with source preferences:**

```rust
// Before (using source field in Citation)
let citations = vec![/* citations with source field */];
let deduplicator = Deduplicator::new().with_config(config);
let groups = deduplicator.find_duplicates(&citations).unwrap();

// After (using external source mapping)
let citations = vec![/* citations without source field */];
let sources = vec!["PubMed", "CrossRef"]; // source for each citation
let deduplicator = Deduplicator::new().with_config(config);
let groups = deduplicator.find_duplicates_with_sources(&citations, &sources).unwrap();
```

## [0.2.4] - 2025-06-11

### Fixed

- Fixed the line continuation for `TI` and `AB` tags in PubMed parser to handle cases where these tags are split across multiple lines

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
