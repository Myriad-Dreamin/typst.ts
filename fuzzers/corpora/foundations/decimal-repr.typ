
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `repr` function with decimals.
#test(repr(decimal("12.0")), "decimal(\"12.0\")")
#test(repr(decimal("3.14")), "decimal(\"3.14\")")
#test(repr(decimal("1234567890.0")), "decimal(\"1234567890.0\")")
#test(repr(decimal("0123456789.0")), "decimal(\"123456789.0\")")
#test(repr(decimal("0.0")), "decimal(\"0.0\")")
#test(repr(decimal("-0.0")), "decimal(\"0.0\")")
#test(repr(decimal("-1.0")), "decimal(\"-1.0\")")
#test(repr(decimal("-9876543210.0")), "decimal(\"-9876543210.0\")")
#test(repr(decimal("-0987654321.0")), "decimal(\"-987654321.0\")")
#test(repr(decimal("-3.14")), "decimal(\"-3.14\")")
#test(repr(decimal("-3.9191919191919191919191919195")), "decimal(\"-3.9191919191919191919191919195\")")
#test(repr(decimal("5.0000000000")), "decimal(\"5.0000000000\")")
#test(repr(decimal("4.0") - decimal("8.0")), "decimal(\"-4.0\")")