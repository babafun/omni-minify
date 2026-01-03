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

## Goals
- Plugin-based architecture
- Safe vs aggressive optimisation
- Format-aware stats

## How to Run

Currently, as there are no releases yet, you will have to compile and run the code from source.
How to do this:
1. Clone the repository onto your computer.
2. Get the file you want minified in the same folder as this README.
3. Ensuring you have Rust installed, in the root folder `omni-minify`, run:
```bash
cargo run -p omni-minify-cli -- {file path}
```

Example test:
`cargo run -p omni-minify-cli -- tests/samples/javascript/bloated.js --stats`
Output will appear in tests/samples/javascript/bloated.min.js