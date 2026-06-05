
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `repr` function with floats.
#test(repr(12.0), "12.0")
#test(repr(3.14), "3.14")
#test(repr(1234567890.0), "1234567890.0")
#test(repr(0123456789.0), "123456789.0")
#test(repr(0.0), "0.0")
#test(repr(-0.0), "-0.0")
#test(repr(-1.0), "-1.0")
#test(repr(-9876543210.0), "-9876543210.0")
#test(repr(-0987654321.0), "-987654321.0")
#test(repr(-3.14), "-3.14")
#test(repr(4.0 - 8.0), "-4.0")
#test(repr(float.inf), "float.inf")
#test(repr(-float.inf), "-float.inf")
#test(repr(float.nan), "float.nan")