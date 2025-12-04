# Changelog

All notable changes to the `dom_sanitizer` crate will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)

## [Unreleased]

### Changed
- Updated `dom_query` dependency version from 0.23.1 to 0.24.0
- Updated `html5ever` dependency version from 0.35.0 to 0.36.1
- Minor code refactoring.


## [0.4.0] - 2025-10-20

### Added
- Re-exported `dom_query` crate for convenience (by @michael-eddy).

### Changed
- Updated `dom_query` dependency version from 0.22.0 to 0.23.1

## [0.3.0] - 2025-09-07

### Changed
- Updated `dom_query` dependency version from 0.20.1 to 0.22.0
- Set MSRV to 1.75.

## [0.2.3] - 2025-08-07

### Changed
- Updated `dom_query` dependency version from 0.19.2 to 0.20.1

## [0.2.2] - 2025-07-08

### Changed
- Updated `dom_query` dependency version from 0.19.1 to 0.19.2
- Updated `html5ever` dependency version from 0.31.0 to 0.35.0

## [0.2.1] - 2025-05-22

### Changed
- Updated `dom_query` dependency to version 0.19.1

## [0.2.0] - 2025-05-17

### Added
- Implemented the `Policy::sanitize_selection` and `PluginPolicy::sanitize_selection` methods to sanitize only nodes within the selected set of nodes.

### Fixed
- Fixed incorrect node retrieval in `next_child_or_sibling`, except for the root node.