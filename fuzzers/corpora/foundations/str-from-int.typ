
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `str` function with integers.
#test(str(12), "12")
#test(str(1234567890), "1234567890")
#test(str(0123456789), "123456789")
#test(str(0), "0")
#test(str(-0), "0")
#test(str(-1), "−1")
#test(str(-9876543210), "−9876543210")
#test(str(-0987654321), "−987654321")
#test(str(4 - 8), "−4")