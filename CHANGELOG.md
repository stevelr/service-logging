# Changelog

## v0.4.7

- set user agent on coralogix calls to crate name & version
- don't set Connection:close

## v0.4.6

- Coralogix config parameters can be &str and don't need to &'static str

## v0.4.5  2021-01-23
- updated dependency to reqwest 0.11

## v0.4.2  2021-01-12
- added silent_logger (logs nothing)

## v0.4.0  2020-12-31

- Breaking change: 
  - Removed prelude module. If you previously imported "service_logging::prelude::*",
    replace it with "service_logging::Logger" to import the trait.

- New features

  - added implementation of ConsoleLogger for non-wasm builds,
    which sends output to stdout using println!.
    The most likely use for this is testing.


