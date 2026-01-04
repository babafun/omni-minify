# TODO: Fix Whitespace Removal to Preserve Quotes

## Plan:
1. [ ] Fix HTML plugin to preserve whitespace inside quotes
2. [ ] Fix CSS plugin to preserve whitespace inside strings  
3. [ ] Fix JavaScript plugin to preserve whitespace inside strings
4. [ ] Run tests to verify no regressions
5. [ ] Add tests for quote preservation

## Implementation Notes:
- Remove all tabs and linebreaks NOT in quotes
- Preserve whitespace within single or double quotes
- Preserve whitespace within template literals for JS
- Remove whitespace surrounding commas but preserve comma whitespace inside quotes

## Testing:
- Verify whitespace inside strings is preserved
- Verify whitespace outside strings is removed
- Verify comma spacing works correctly outside quotes only
