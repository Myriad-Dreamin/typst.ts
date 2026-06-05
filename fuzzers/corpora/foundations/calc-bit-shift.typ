
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `bit-lshift` and `bit-rshift` functions.
#test(32.bit-lshift(2), 128)
#test(694.bit-lshift(0), 694)
#test(128.bit-rshift(2), 32)
#test(128.bit-rshift(12345), 0)
#test((-7).bit-rshift(2), -2)
#test((-7).bit-rshift(12345), -1)
#test(128.bit-rshift(2, logical: true), 32)
#test((-7).bit-rshift(61, logical: true), 7)
#test(128.bit-rshift(12345, logical: true), 0)
#test((-7).bit-rshift(12345, logical: true), 0)