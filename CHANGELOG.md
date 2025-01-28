# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
