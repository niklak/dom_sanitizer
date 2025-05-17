# Changelog

All notable changes to the `dom_sanitizer` crate will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)

## [0.2.0] - 2025-05-17

### Added
- Implemented the `Policy::sanitize_selection` and `PluginPolicy::sanitize_selection` methods to sanitize only nodes within the selected set of nodes.

### Fixed
- Fixed incorrect node retrieval in `next_child_or_sibling`, except for the root node.