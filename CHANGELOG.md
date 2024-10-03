# 0.3.1

- Added support for Launchpad Mini MK3
- Added `Button80::grid()` and `Button80::control()` convenience functions

# 0.3.0

- Updated midir to 0.9.1
- Added serde support behind a `serde` feature flag
  - Only for Color and Pad structs, currently
- `impl Eq for Color`
- `impl Sub<Pad> for Pad`
