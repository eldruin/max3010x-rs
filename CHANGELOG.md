# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## [Unreleased] - ReleaseDate

## [0.2.0] - 2024-12-02

### Changed
- [breaking-change] Update `embedded-hal` to version 1.0.0
- [breaking-change] Renamed `mode::MultiLED` mode marker `mode::MultiLed` due to naming conventions.
- MSRV bumped to 1.62

### Fixed
- Masking reserved bitfields for FIFO write/read pointer and overflow registers.

## 0.1.0 - 2019-03-10

This is the initial release of the driver to crates.io. All changes will
be documented in this CHANGELOG.

<!-- next-url -->
[Unreleased]: https://github.com/eldruin/max3010x-rs/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/eldruin/max3010x-rs/compare/v0.1.0...v0.2.0
