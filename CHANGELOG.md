# Changelog

All notable changes to the `dom_sanitizer` crate will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)

## [0.2.1] - 2025-07-08

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