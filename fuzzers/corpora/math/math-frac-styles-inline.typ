
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test inline layout of styled fractions
#set math.frac(style: "horizontal")
$a/(b+c), frac(a, b+c, style: "skewed"), frac(a, b+c, style: "vertical")$