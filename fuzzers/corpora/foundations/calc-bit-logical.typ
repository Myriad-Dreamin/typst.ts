
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `bit-not`, `bit-and`, `bit-or` and `bit-xor` functions.
#test(64.bit-not(), -65)
#test(0.bit-not(), -1)
#test((-56).bit-not(), 55)
#test(128.bit-and(192), 128)
#test(192.bit-and(224), 192)
#test((-1).bit-and(325532), 325532)
#test(0.bit-and(-53), 0)
#test(0.bit-or(-1), -1)
#test(5.bit-or(3), 7)
#test((-50).bit-or(3), -49)
#test(64.bit-or(32), 96)
#test((-1).bit-xor(1), -2)
#test(64.bit-xor(96), 32)
#test((-1).bit-xor(-7), 6)
#test(0.bit-xor(492), 492)