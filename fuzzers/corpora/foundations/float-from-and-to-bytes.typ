
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test float `from-bytes()` and `to-bytes()`.
#test(float.from-bytes(bytes((0, 0, 0, 0, 0, 0, 240, 63))), 1.0)
#test(float.from-bytes(bytes((63, 240, 0, 0, 0, 0, 0, 0)), endian: "big"), 1.0)
#test(1.0.to-bytes(), bytes((0, 0, 0, 0, 0, 0, 240, 63)))
#test(1.0.to-bytes(endian: "big"), bytes((63, 240, 0, 0, 0, 0, 0, 0)))

#test(float.from-bytes(bytes((0, 0, 32, 64))), 2.5)
#test(float.from-bytes(bytes((64, 32, 0, 0)), endian: "big"), 2.5)
#test(2.5.to-bytes(size: 4), bytes((0, 0, 32, 64)))
#test(2.5.to-bytes(size: 4, endian: "big"), bytes((64, 32, 0, 0)))