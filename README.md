# omni-minify

**Minify literally anything.**

A format-aware, open-source CLI that optimises text, code, and binary assets:
- JS / CSS / HTML
- Rust / C / C++ / Java
- 3D models (GLB) via mesh optimisation

This is not a formatter.
This is not a zip file.

This is *semantic size reduction*, so that once you're done with your code (or GLB file), you free up a bunch of space, save download time for clients connecting to your website, or optimise your mesh to run quickly in your game.

## Status
🚧 Early development - do not use on code you care about without backing it up.

Currently known bugs:
- It removes whitespace from text in quotes/paragraphs at times (universal)

## Goals
- Plugin-based architecture
- Safe vs aggressive optimisation
- Format-aware stats

## Contributing
Contributing is quite easy! All you have to do is fork the repository, create a new plugin in crates/plugins/(supported filetype)/, and edit the CLI to connect to it.

## Plugin versions
Current plugin versions and states:
- JavaScript is at v0.1.0 - currently in beta
- CSS is at v1.0.0 - stable for usage in production
- HTML is at v0.1.0 - currently in beta
- GLB currently has an empty plugin - will be worked on!
## How to Run

Currently, as there are no releases yet, you will have to compile and run the code from source.
How to do this:
1. Clone the repository onto your computer.
2. take your terminal into the repository, e.g:
```bash
cd omni-minify
```
3. Get the file you want minified in the same folder as this README.
4. Ensuring you have Rust installed (if you do not, please install by following the instructions from [the official Rust installer](https://rustup.rs/)), in the root folder `omni-minify`, run:
```bash
cargo cli {file path}
```

Example test:
`cargo cli tests/samples/javascript/bloated.js --stats`
Output will appear in tests/samples/javascript/bloated.min.js

## Building

This project supports three profiles to build with:

1. Debug (or dev) profile (`cargo build`) - this builds the default dev profile, with a small amount of optimisation. Keeps Rust's standard debug information
2. Release profile (`cargo build -r`) - this builds an aggressively speed-optimised release. This is the one that will be released on the GitHub releases page.
3. Mini profile (`cargo mini`) - this builds a size-optimised release. Use this if you want your binary to take as little space as possible.
