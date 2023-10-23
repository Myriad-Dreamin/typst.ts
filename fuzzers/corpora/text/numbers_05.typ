
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test the `repr` function with integers.
#repr(12) \
#repr(1234567890) \
#repr(0123456789) \
#repr(0) \
#repr(-0) \
#repr(-1) \
#repr(-9876543210) \
#repr(-0987654321) \
#repr(4 - 8)
