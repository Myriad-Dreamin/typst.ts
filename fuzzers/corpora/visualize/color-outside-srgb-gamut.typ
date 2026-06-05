
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Colors outside the sRGB gamut.
#box(square(size: 9pt, fill: oklab(90%, -0.2, -0.1)))
#box(square(size: 9pt, fill: oklch(50%, 0.5, 0deg)))