
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test alpha modification.
#test-repr(luma(100%, 100%).transparentize(50%), luma(100%, 50%))
#test-repr(luma(100%, 100%).transparentize(75%), luma(100%, 25%))
#test-repr(luma(100%, 50%).transparentize(50%), luma(100%, 25%))
#test-repr(luma(100%, 10%).transparentize(250%), luma(100%, 0%))
#test-repr(luma(100%, 40%).transparentize(-50%), luma(100%, 70%))
#test-repr(luma(100%, 0%).transparentize(-100%), luma(100%, 100%))