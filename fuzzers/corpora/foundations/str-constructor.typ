
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test conversion to string.
#test(str(123), "123")
#test(str(123, base: 3), "11120")
#test(str(-123, base: 16), "−7b")
#test(str(9223372036854775807, base: 36), "1y2p0ij32e8e7")
#test(str(50.14), "50.14")
#test(str(10 / 3).len() > 10, true)