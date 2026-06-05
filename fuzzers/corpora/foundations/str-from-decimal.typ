
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `str` function with decimals.
#test(str(decimal("12")), "12")
#test(str(decimal("12.0")), "12.0")
#test(str(decimal("3.14")), "3.14")
#test(str(decimal("1234567890.0")), "1234567890.0")
#test(str(decimal("0123456789.0")), "123456789.0")
#test(str(decimal("0.0")), "0.0")
#test(str(decimal("-0.0")), "0.0")
#test(str(decimal("-1.0")), "−1.0")
#test(str(decimal("-9876543210.0")), "−9876543210.0")
#test(str(decimal("-0987654321.0")), "−987654321.0")
#test(str(decimal("-3.14")), "−3.14")
#test(str(decimal("-3.9191919191919191919191919195")), "−3.9191919191919191919191919195")
#test(str(decimal("5.0000000000")), "5.0000000000")
#test(str(decimal("4.0") - decimal("8.0")), "−4.0")
#test(str(decimal("4") - decimal("8")), "−4")