# CHANGE LOG FOR RSLIFE PACKAGE

## [0.2.1] - 2024-08-06

### Added

- Support for custom SOA SULT table generation using Makeham's Law.

## [0.2.0] - 2024-08-04

### Added

- Robust logic to ensure only the first row with `qx == 1.0` is kept in mortality tables.
- Improved documentation for all major modules and public functions, including actuarial notation and usage examples.
- Debug print utilities in tests to inspect DataFrame structure and values.
- Comprehensive validation for DataFrame schema and mortality data.

### Changed

- Updated module structure documentation to reflect actual folder and file organization.
- Improved error handling and reporting for missing or malformed mortality data.

### Fixed

- Fixed doctest and documentation issues for builder-pattern functions and result types.

### Removed

- Removed unused dependencies and cleaned up legacy code paths.

## [0.1.3] - 2025-07-25

### Changed in v0.1.3

- **Unified API Interface**: Rationalized function signatures across all modules with clear, consistent parameter requirements for enhanced usability.
- **Improved Configuration Integration**: Enhanced `config` and argument integration throughout the library, making the API more intuitive and reducing repetitive code patterns.
- **Enhanced Documentation**: Updated function documentation to reflect the unified API approach and improved clarity for developers.

### Fixed

- Consistent parameter ordering across all actuarial functions for better developer experience.
- Streamlined function exports and module organization.

### Note

- This version continues the API stabilization begun in v0.1.2, focusing on unifying the interface patterns across all function categories.

## [0.1.2] - 2025-07-23

### Changed in v0.1.2

- Major API update: all actuarial, survival, and selection functions now require explicit `config` and argument parameters (e.g., `Ax(config, x)`, `tpx(config, t, x)`, `tpx_(config, entry_age, t, x)`).
- Selection functions consistently use a `_` suffix and require an `entry_age` argument.
- Function signatures and documentation updated for clarity and discoverability.
- Module-level and crate-level documentation improved and standardized.
- Export structure reorganized for easier use and better grouping in `prelude` and top-level modules.

### Fixed

- Improved error messages and validation for selection and fractional functions.
- Doc examples and module docs now pass `cargo test --doc`.

### Note

- This release introduces breaking changes to function signatures and exports. Please update your code to use the new explicit argument conventions. See the README and documentation for migration guidance.

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2025-07-20

### Changed in v0.1.1

- **BREAKING**: Updated annuity function naming convention to follow standard actuarial notation
  - `axn_due` → `aaxn`
  - `tax_due` → `taax`
  - `taxn_due` → `taaxn`
  - `Iax_due` → `Iaax`
  - `Iaxn_due` → `Iaaxn`
  - `tIax_due` → `tIaax`
  - `tIaxn_due` → `tIaaxn`
  - `gIax_due` → `gIaax`
  - `gIaxn_due` → `gIaaxn`

### Updated

- All documentation and examples to reflect new function names
- Function exports in prelude module
- Test files to use new naming convention
- README.md with comprehensive function documentation organized by categories

### Migration Guide

If upgrading from 0.1.0, update your function calls:

```rust
// Old (0.1.0)
let annuity = axn_due(&config, 65, 20, 1)?;
let deferred = tax_due(&config, 65, 10, 1)?;

// New (0.1.1)
let annuity = aaxn(&config, 65, 20, 1)?;
let deferred = taax(&config, 65, 10, 1)?;
```

## [0.1.0] - 2025-07-20

### Added

- Initial release of RSLife actuarial library
- Comprehensive mortality table calculations
- XML parsing for Society of Actuaries mortality data
- Life insurance functions (Ax, Axn, AExn, etc.)
- Annuity calculations with multiple payment frequencies
- Fractional age calculations with UDD, CFM, and HPB assumptions
- Performance optimization with 4-level detail system
- Polars DataFrame integration for efficient data processing
- Full prelude module for convenient imports

### Features

- Standard actuarial notation following industry conventions
- Support for geometric and arithmetic increasing benefits
- Deferred insurance and annuity calculations
- Survival and mortality probability functions
- Well-documented API with mathematical formulations
- Integration tests ensuring function reliability
