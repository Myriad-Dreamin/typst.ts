
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// In this bug, empty-range glyphs at line break boundaries could be duplicated.
// This happens for Thai specifically because it has both
// - line break opportunities
// - shaping that results in multiple glyphs in the same cluster
#set text(font: "Noto Sans Thai")
#h(85pt) งบิก