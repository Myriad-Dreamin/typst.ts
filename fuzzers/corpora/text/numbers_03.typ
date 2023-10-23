
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test the `str` function with integers.
#str(12) \
#str(1234567890) \
#str(0123456789) \
#str(0) \
#str(-0) \
#str(-1) \
#str(-9876543210) \
#str(-0987654321) \
#str(4 - 8)
