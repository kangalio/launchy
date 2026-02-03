# 0.4.0

- Added support for Launchpad MK1 (https://github.com/kangalio/launchy/pull/17, thanks @sapphicdisaster)

New major release because this project isn't fresh in my memory and I can't guarantee for technically breaking changes not having slipped by

# 0.3.1

- Added support for Launchpad Mini MK3 (https://github.com/kangalio/launchy/pull/14, thanks @rix0rrr)
- Added `Button80::grid()` and `Button80::control()` convenience functions (https://github.com/kangalio/launchy/pull/14, thanks @rix0rrr)

# 0.3.0

- Updated midir to 0.9.1
- Added serde support behind a `serde` feature flag
  - Only for Color and Pad structs, currently
- `impl Eq for Color`
- `impl Sub<Pad> for Pad`
