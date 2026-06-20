
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test fallback with grapheme clusters.
#let bird = symbol("🐦‍⬛")
#bird or 🐦‍⬛
$bird "or" 🐦‍⬛$

#set text(font: "Noto Color Emoji")
#show math.equation: set text(font: "Noto Color Emoji")
#bird or 🐦‍⬛
// Warning: 1-16 current font is not designed for math
// Hint: 1-16 rendering may be poor
$bird "or" 🐦‍⬛$