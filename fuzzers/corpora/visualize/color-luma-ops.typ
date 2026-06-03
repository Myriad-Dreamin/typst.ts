
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test gray color modification.
#test-repr(luma(20%).lighten(50%), luma(60%))
#test-repr(luma(80%).darken(20%), luma(64%))
#test-repr(luma(80%).negate(space: luma), luma(20%))