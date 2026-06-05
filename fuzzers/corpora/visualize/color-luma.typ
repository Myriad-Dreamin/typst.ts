
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test gray color conversion.
#stack(dir: ltr, rect(fill: luma(0)), rect(fill: luma(80%)))