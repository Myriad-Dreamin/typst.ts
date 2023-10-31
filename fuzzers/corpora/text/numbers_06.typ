
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test the `repr` function with floats.
#repr(12.0) \
#repr(3.14) \
#repr(1234567890.0) \
#repr(0123456789.0) \
#repr(0.0) \
#repr(-0.0) \
#repr(-1.0) \
#repr(-9876543210.0) \
#repr(-0987654321.0) \
#repr(-3.14) \
#repr(4.0 - 8.0)
