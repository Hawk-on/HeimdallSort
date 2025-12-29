# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.2.0] - 2025-12-29

### Added
- **Video Support**: Full support for `.mp4`, `.mov`, `.avi`, `.mkv`. Includes automated thumbnail generation using bundling FFmpeg.
- **Sidecar Support**: Operations (move, rename, delete) now automatically handle associated `.xmp`, `.json`, and `.aae` files.
- **Visual Comparison**: New side-by-side comparison tool for reviewing duplicate candidates effectively.
- **Hybrid Duplicate Detection**: Pipeline now uses a fast pre-filter (size + partial hash) before running perceptual hashing, significantly improving speed.

### Changed
- **UI Overhaul**: Refactored the entire CSS codebase into a modular architecture.
- **Theme**: Introduced "Premium Dark" theme with improved typography (Cinzel Decorative headers).
- **Notifications**: Replaced native browser alerts with a custom, non-intrusive Toast notification system.
- **Sorting**: "Strict Sorting" enabled. Options to fallback to file modification time have been removed to prevent false positives. Images without EXIF are specifically moved to an "Uten dato" folder.

### Fixed
- **Large Binaries**: Removed large FFmpeg binaries from Git history to resolve GitHub push size limits (replaced with `npm run setup` script).
- **Compilation**: Fixed various Rust compilation warnings and `rexif` version conflicts (downgraded to safe full-decode fallback temporarily).

## [1.1.0] - 2025-12-28
- Initial Release features (assumed)
