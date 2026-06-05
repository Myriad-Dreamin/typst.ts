
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Warning: 2-36 `show par: set block(spacing: ..)` has no effect anymore
// Hint: 2-36 this is specific to paragraphs as they are not considered blocks anymore
// Hint: 2-36 write `set par(spacing: ..)` instead
#show par: set block(spacing: 12pt)