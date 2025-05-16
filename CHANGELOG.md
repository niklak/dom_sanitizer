# Changelog

All notable changes to the `dom_sanitizer` crate will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)

## [Unreleased]

### Fixed
- Fixed incorrect node retrieval in `next_child_or_sibling`, except for the root node.