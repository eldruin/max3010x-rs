# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- MSRV bumped to 1.60 (required by `embedded-hal` 1.0.0)
- [breaking-change] Update `embedded-hal` to version 1.0.0
- [breaking-change] Renamed `mode::MultiLED` mode marker `mode::MultiLed` due to naming conventions.

### Fixed
- Masking reserved bitfields for FIFO write/read pointer and overflow registers.

## 0.1.0 - 2019-03-10

This is the initial release of the driver to crates.io. All changes will
be documented in this CHANGELOG.

[Unreleased]: https://github.com/eldruin/max3010x-rs/compare/v0.1.0...HEAD
