
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test the `str` function with floats.
#str(12.0) \
#str(3.14) \
#str(1234567890.0) \
#str(0123456789.0) \
#str(0.0) \
#str(-0.0) \
#str(-1.0) \
#str(-9876543210.0) \
#str(-0987654321.0) \
#str(-3.14) \
#str(4.0 - 8.0)
