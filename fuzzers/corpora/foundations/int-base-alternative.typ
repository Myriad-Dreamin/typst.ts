
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test numbers with alternative bases.
#test(0x10, 16)
#test(0b1101, 13)
#test(0xA + 0xa, 0x14)