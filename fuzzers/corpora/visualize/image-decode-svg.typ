// SKIP: Temporarily removed for Typst 0.15.0-rc1 corpus compatibility review.

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test parsing from svg data
// Warning: 8-14 `image.decode` is deprecated, directly pass bytes to `image` instead
// Hint: 8-14 it will be removed in Typst 0.15.0
#image.decode(`<svg xmlns="http://www.w3.org/2000/svg" height="140" width="500"><ellipse cx="200" cy="80" rx="100" ry="50" style="fill:yellow;stroke:purple;stroke-width:2" /></svg>`.text, format: "svg")
