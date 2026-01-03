# omni-minify CLI Todo

## High-Level Tasks

- [x] Create requirements specification
- [x] Create design specification  
- [x] Create implementation task list
- [x] Implement core crate (traits, enums, stats)
- [x] Implement CLI crate (argument parsing, orchestration)
- [x] Implement plugin detection system
- [ ] Implement JavaScript plugin
- [ ] Implement CSS plugin
- [ ] Implement HTML plugin
- [ ] Add statistics reporting
- [ ] Test CLI on sample inputs
- [ ] Document usage and examples

## Recent Progress

### ✅ Plugin Detection System (Task 3)
- **Enhanced plugin registry** with robust detection engine
- **Implemented deterministic plugin selection** (first match wins)
- **Added format override functionality** for `--format` flag
- **Improved error handling** for unsupported formats
- **Added comprehensive property-based testing** for detection consistency
- **Enhanced CLI error messages** with helpful suggestions

### ✅ Core Infrastructure Complete
- **Minifier trait system** with MinifyLevel, MinifyStats, and MinifyError
- **CLI argument parsing** with clap (--safe, --aggressive, --stats, --format flags)
- **Plugin registry architecture** ready for plugin implementations
- **File I/O operations** with proper error handling
- **Property-based testing framework** using quickcheck

## Next Steps

The foundation is now complete. Ready to implement actual minification plugins:

1. **JavaScript Plugin** - Content detection + SWC-based minification
2. **CSS Plugin** - Content detection + lightningcss-based minification  
3. **HTML Plugin** - Content detection + custom parser-based minification

Each plugin will integrate seamlessly with the existing detection system.