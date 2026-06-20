
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// The different color spaces.
#let col = rgb(50%, 64%, 16%)
#box(square(size: 9pt, fill: col))
#box(square(size: 9pt, fill: rgb(col)))
#box(square(size: 9pt, fill: oklab(col)))
#box(square(size: 9pt, fill: oklch(col)))
#box(square(size: 9pt, fill: luma(col)))
#box(square(size: 9pt, fill: cmyk(col)))
#box(square(size: 9pt, fill: color.linear-rgb(col)))
#box(square(size: 9pt, fill: color.hsl(col)))
#box(square(size: 9pt, fill: color.hsv(col)))