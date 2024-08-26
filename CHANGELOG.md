# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.6.0] - 2024-08-26

### Added

- Add new parameters to `bss_check.printSymbolComparison`.
  - `printGoods`: Allows toggling printing the GOOD symbols.
  - `printingStyle`: The style to use to print the symbols, either `"csv"` or
    `"listing"`.
  - TODO: port to Rust.
- New `MapFile.fixupNonMatchingSymbols` method.
  - Allows to fixup size calculation of symbols messed up by symbols with the
    same name suffixed with `.NON_MATCHING`.
- Add support for `.NON_MATCHING` suffix on symbols on progress calculation.
  - If a symbol exists and has a `.NON_MATCHING`-suffixed counterpart then
    consider it not mateched yet.

### Changed

- Minor cleanups.

## [2.5.1] - 2024-08-09

### Fixed

- Fix Github Action file.

## [2.5.0] - 2024-08-09

### Added

- Add Minimal Supported Rust Version (MSRV) to Cargo.toml.
- Add `MapFile::new_from_map_file` function to simplify `MapFile` creation.
- Add `serde` feature to the Rust crate.
  - Allows serializing and deserializing a `MapFile` object using serde.

### Changed

- Tweak symbol comparison logic a bit.
  - Symbol shifting (due to different sizes or extra/missing symbols) should
    not affect comparing non shifted files.
- `Cargo.lock` file is now committed to the repo.
- Change Rust functions to properly take references instead of consuming the
  argument.

### Fixed

- Fix `MapFile::find_lowest_differing_symbol` not returning a previous symbol
  from a previous file if the symbol found is the first symbol from the file.

## [2.4.0] - 2024-03-25

### Added

- Add `endian` argument to `doFirstDiff`.
- Add `--endian` option to `first_diff` script.

### Removed

- Dropped Python 3.7 support.
  - Python 3.8 is the minimum supported version now.

## [2.3.7] - 2024-02-27

### Fixed

- Fix not recognizing file entries which are splited in two lines because its
  section name was too long to fit.

## [2.3.6] - 2024-02-23

### Added

- Add issue templates for bug reports and feature suggestions.

### Fixed

- Fix not recognizing sections that don't start with dots (`.`).

## [2.3.5] - 2024-02-04

### Fixed

- Fix `MapFile.compareFilesAndSymbols` reporting the wrong address as the
  expected address.

## [2.3.4] - 2024-01-26

### Changed

- Frontend scripts now give a better error if the mapfile does not exist.

## [2.3.2] - 2024-01-17

### Added

- Add optional `categoryColumnSize` parameter to `ProgressStats.getHeaderAsStr`
  and `ProgressStats.getEntryAsStr`.
  - Allows to set the size of the first column.

### Fixed

- Fix Rust's implementation of `File` not returning a `pathlib.Path` object for
  the `filepath` member.

## [2.3.1] - 2023-12-23

### Added

- Add a few utility methods to `ProgressStats`.

### Changed

- `pyo3` is no longer needed to use this crate as Rust-only library.
- Updated Rust dependencies.

## [2.3.0] - 2023-11-05

### Added

- Support for parsing clang lld's map fles.
- New functions:
  - `MapFile.parseMapContents`/`MapFile::parse_map_contents`
    - Parses the map contents passed as the argument, without requiring the map
      being on an actual file.
    - The map format will be guessed on the contents. Currently both the GNU ld
      and clang ld.lld map formats are recognized.
  - `MapFile.parseMapContentsGNU`/`MapFile::parse_map_contents_gnu`
    - Parses the map contents passed as the argument, without requiring the map
      being on an actual file.
    - This function only parses the GNU ld map format.
  - `MapFile.parseMapContentsLLD`/`MapFile::parse_map_contents_lld`
    - Parses the map contents passed as the argument, without requiring the map
      being on an actual file.
    - This function only parses the clang ld.lld map format.
- New members:
  - `Symbol.align`/`Symbol::align`, `File.align`/`File::align` and
    `Segment.align`/`Segment::align`: The alignment the given type. This member
    will be filled by the parser only if the mapfile provides this information.

### Changed

- `MapFile.readMapFile`/`MapFile::read_map_file` can now guess the map format
  between any of the known formats.
- Some known symbol names will be automatically filtered out during the parsing
  step.
  - Currently only `gcc2_compiled.` is filtered out.

### Fixed

- Fix parser not detecting `*fill*` lines on GNU ld maps if they specified the
  value that was used for filling/padding.
- `.sbss`, `COMMON` and `.scommon` sections are now properly considered noload
  sections.

## [2.2.1] - 2023-10-08

### Fixed

- Fix Rust crate size being too big
  - crates.io was rejecting the package because of the size
  - Cargo was packaging all the map files and test cases, making the package be
    15 MiB. Now it is around 16.3 KiB

## [2.2.0] - 2023-10-08

### Added

- Add new Rust re-implementation (#15)
  - Allows using this library in native Rust projects
  - It does not replace the Python implementation due to restrictions on how
    Rust bindings work.
  - Python bindings for the Rust implementation exists, but they are not used
    or exposed to the user
- Now this library has a Rust crate.
  - Check it at <https://crates.io/crates/mapfile_parser>

### Changed

- Speed-up the actual parsing of a mapfile by using the native Rust implementation
  - Up to 10 times faster parsing has been registered
- Change build system from `hatchling` to `maturin`

## [2.1.5] - 2023-10-02

### Added

- Add `CHANGELOG.md`
- Add markdown linter to CI

### Changed

- Cleanup the `README.md` a bit
- Update Github Action's dependencies

### Deprecated

- Deprecate `File.getName`
  - The method itself doesn't make sense, instead operate on `File.filepath` directly
- Deprecate `MapFile.debugging`
- Deprecate `progress` frontend's `--debugging` flag

## [2.1.4] - 2023-09-11

### Fixed

- Fix vrom calculation if the first symbol of a file is not available in the
  map file

## [1.3.2] - 2023-09-11

### Added

- Add version info to the cli
- Add small testing suite
- Add machine-friendly/non-human-readable option for json generation.
  - Numbers are outputted as real numbers instead of prettified strings

### Changed

- Allow csv conversion to be written to a file instead of only printing to stdout
- Output `none` instead of `"None"` for symbols with no vrom when generating
  json output.

### Fixed

- Fix vrom calculation if the first symbol of a file is not available in the
  map file

Full changes: <https://github.com/Decompollaborate/mapfile_parser/compare/702a73452059ce4e97cda011e09dc4ef2a7b9dec...ba444b0bbfdfad7fb07347bf656b7fd4381596fb>

## [2.1.3] - 2023-08-30

### Fixed

- Fix version number
  - pypi thought previous version was a prerelease instead of a full release

## [2.1.2] - 2023-08-30

### Added

- Add machine-friendly/non-human-readable option for json generation.
  - Numbers are outputted as real numbers instead of prettified strings
- Add some CI tests

### Changed

- Output `none` instead of `"None"` for symbols with no vrom when generating
  json output.
- Make dummy files for `*fill*` lines instead of adding them to the previous file.
- Don't drop the dummy segment if it actually has data on it

## [2.1.1] - 2023-08-15

### Fixed

- Fix off-by-one issue which was throwing away tons of data

## [2.1.0] - 2023-08-14

### Added

- Add `bss_check` frontend
  - Allows to search for reordered bss variables by comparing two map files.

### Changed

- Don't skip important lines in some kinds of map files.
  - This may produce map parsing to be a bit slower but it should work properly
    with more kinds of mapfiles

### Fixed

- Try to prevent crashes if a file is found before the first segment is found

## [2.0.1] - 2023-08-07

### Changed

- Makes `Symbol`, `File` and `Segment` hashable

## [1.3.1] - 2023-08-06

### Changed

- Make `Symbol` and `File` types hashable

## [2.0.0] - 2023-08-01

### Added

- `toJson` method which allow serializing map files into the json format.
- `jsonify` frontend which allows converting a mapfile into a json format from
  the CLI.

### Changed

- Change logic of `MapFile` so it can parse and organize each file in proper
  segments.
  - This breaks old ways of iterating the `MapFile` class. Now iterating it
    yields a `Segment`, iterating that yields a `File`.
- Rename `segmentType` to `sectionType` and `filterBySegmentType` to
  `filterBySectionType`.

## [1.3.0] - 2023-08-01

### Added

- Add function `toJson` to export map file as a json. It returns a `dict`.
- New `jsonify` frontend, which allows converting a map file in the cli.

## [1.2.1] - 2023-07-28

### Fixed

- Fix missing `:` colon even when passing `addColons=True` to the `first_diff`
  frontend

## [1.2.0] - 2023-07-28

### Added

- `first_diff` frontend:
  - Allow an optional bytes converter callback. It can be useful to perform
    analysis or instruction decoding on the caller side.
  - Parameter to toggle colons (`:`) in bytes difference output

## [1.1.5] - 2023-07-28

### Fixed

- Fix map parsing ignoring some `*fill*` entries
- Improve symbol info output a bit

## [1.1.4] - 2023-04-03

### Fixed

- Add missing `request` requirement

## [1.1.3] - 2023-02-22

### Added

- Add flag to enable debug prints
- Add flag to specify the path index

### Fixed

- Properly handle files with multiple extensions

## [1.1.2] - 2022-12-14

### Changed

- Modularize `upload_frogress` frontend

## [1.1.1] - 2022-12-14

### Added

- Add examples to README

## [1.1.0] - 2022-12-14

### Added

- Compute vrom addresses of symbols and files
- Provide various front-ends clis (see README for more info):
  - `first_diff`
  - `pj64_syms`
  - `progress`
  - `sym_info`
  - `symbol_sizes_csv`
  - `upload_frogress`

[Full changelog](https://github.com/Decompollaborate/mapfile_parser/compare/1.0.0...1.1.0)

## [1.0.0] - 2022-12-13

### Added

- Initial release

[unreleased]: https://github.com/Decompollaborate/mapfile_parser/compare/master...develop
[2.6.0]: https://github.com/Decompollaborate/mapfile_parser/compare/2.5.1...2.6.0
[2.5.1]: https://github.com/Decompollaborate/mapfile_parser/compare/2.5.0...2.5.1
[2.5.0]: https://github.com/Decompollaborate/mapfile_parser/compare/2.4.0...2.5.0
[2.4.0]: https://github.com/Decompollaborate/mapfile_parser/compare/2.3.7...2.4.0
[2.3.7]: https://github.com/Decompollaborate/mapfile_parser/compare/2.3.6...2.3.7
[2.3.6]: https://github.com/Decompollaborate/mapfile_parser/compare/2.3.5...2.3.6
[2.3.5]: https://github.com/Decompollaborate/mapfile_parser/compare/2.3.4...2.3.5
[2.3.4]: https://github.com/Decompollaborate/mapfile_parser/compare/2.3.2...2.3.4
[2.3.2]: https://github.com/Decompollaborate/mapfile_parser/compare/2.3.1...2.3.2
[2.3.1]: https://github.com/Decompollaborate/mapfile_parser/compare/2.3.0...2.3.1
[2.3.0]: https://github.com/Decompollaborate/mapfile_parser/compare/2.2.1...2.3.0
[2.2.1]: https://github.com/Decompollaborate/mapfile_parser/compare/2.2.0...2.2.1
[2.2.0]: https://github.com/Decompollaborate/mapfile_parser/compare/2.1.5...2.2.0
[2.1.5]: https://github.com/Decompollaborate/mapfile_parser/compare/2.1.4...2.1.5
[2.1.4]: https://github.com/Decompollaborate/mapfile_parser/compare/2.1.3...2.1.4
[1.3.2]: https://github.com/Decompollaborate/mapfile_parser/compare/1.3.1...1.3.2
[2.1.3]: https://github.com/Decompollaborate/mapfile_parser/compare/2.1.2...2.1.3
[2.1.2]: https://github.com/Decompollaborate/mapfile_parser/compare/2.1.1...2.1.2
[2.1.1]: https://github.com/Decompollaborate/mapfile_parser/compare/2.1.0...2.1.1
[2.1.0]: https://github.com/Decompollaborate/mapfile_parser/compare/2.0.1...2.1.0
[2.0.1]: https://github.com/Decompollaborate/mapfile_parser/compare/2.0.0...2.0.1
[1.3.1]: https://github.com/Decompollaborate/mapfile_parser/compare/1.3.0...1.3.1
[2.0.0]: https://github.com/Decompollaborate/mapfile_parser/compare/1.3.0...2.0.0
[1.3.0]: https://github.com/Decompollaborate/mapfile_parser/compare/1.2.1...1.3.0
[1.2.1]: https://github.com/Decompollaborate/mapfile_parser/compare/1.2.0...1.2.1
[1.2.0]: https://github.com/Decompollaborate/mapfile_parser/compare/1.1.5...1.2.0
[1.1.5]: https://github.com/Decompollaborate/mapfile_parser/compare/1.1.4...1.1.5
[1.1.4]: https://github.com/Decompollaborate/mapfile_parser/compare/1.1.3...1.1.4
[1.1.3]: https://github.com/Decompollaborate/mapfile_parser/compare/1.1.2...1.1.3
[1.1.2]: https://github.com/Decompollaborate/mapfile_parser/compare/1.1.1...1.1.2
[1.1.1]: https://github.com/Decompollaborate/mapfile_parser/compare/1.1.0...1.1.1
[1.1.0]: https://github.com/Decompollaborate/mapfile_parser/compare/1.0.0...1.1.0
[1.0.0]: https://github.com/Decompollaborate/mapfile_parser/releases/tag/1.0.0
