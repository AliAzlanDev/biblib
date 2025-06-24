# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0 - Unreleased]

### Removed

- **Citation ID field**: The `id` field has been completely removed from the `Citation` struct as it is not part of the actual bibliographic data parsed from citation formats
- **Source tracking in Citation struct**: The `source` field has been completely removed from the `Citation` struct and all parser implementations
- **Parser `with_source()` methods**: All parsers no longer have the `with_source()` method for setting citation source

### Changed

- **`detect_and_parse()` function signature**: Now takes only one parameter (`content`) instead of two (`content`, `source`)

### Added

- **Enhanced deduplication API**: New `find_duplicates_with_sources()` method for source-aware deduplication when sources are managed externally

### Migration Guide

#### 1. Citation ID Management

If you were relying on the auto-generated `id` field in citations, you'll need to manage IDs at the application level:

**Before (v0.2.x):**

```rust
let parser = RisParser::new();
let citations = parser.parse(input).unwrap();
let citation_id = citations[0].id.clone(); // Auto-generated nanoid
```

**After (v0.3.x):**

```rust
let parser = RisParser::new();
let citations = parser.parse(input).unwrap();
// Generate IDs in your application if needed:
let citation_id = nanoid::nanoid!(); // or your preferred ID system
```

#### 2. Source Tracking

Source tracking must now be handled at the application level instead of within the Citation struct:

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

#### 3. Format Detection API

The `detect_and_parse()` function no longer accepts a source parameter:

**Before (v0.2.x):**

```rust
let (citations, format) = detect_and_parse(content, "PubMed").unwrap();
```

**After (v0.3.x):**

```rust
let (citations, format) = detect_and_parse(content).unwrap();
// Track source separately in your application
```

#### 4. Deduplication with Sources

Use the new `find_duplicates_with_sources()` method when you need source-aware deduplication:

**Before (v0.2.x):**

```rust
let citations = vec![/* citations with source field */];
let deduplicator = Deduplicator::new().with_config(config);
let groups = deduplicator.find_duplicates(&citations).unwrap();
```

**After (v0.3.x):**

```rust
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
