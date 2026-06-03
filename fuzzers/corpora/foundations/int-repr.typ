
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `repr` function with integers.
#test(repr(12), "12")
#test(repr(1234567890), "1234567890")
#test(repr(0123456789), "123456789")
#test(repr(0), "0")
#test(repr(-0), "0")
#test(repr(-1), "-1")
#test(repr(-9876543210), "-9876543210")
#test(repr(-0987654321), "-987654321")
#test(repr(4 - 8), "-4")