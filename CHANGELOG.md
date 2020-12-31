# Changelog

## 0.4

- Breaking change: 
  - Removed prelude module. If you previously imported "service_logging::prelude::*",
    replace it with "service_logging::Logger" to import the trait.

- New features

  - added implementation of ConsoleLogger for non-wasm builds,
    which sends output to stdout using println!.
    The most likely use for this is testing.


