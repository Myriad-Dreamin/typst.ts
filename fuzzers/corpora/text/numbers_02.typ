
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test floats.
#12.0 \
#3.14 \
#1234567890.0 \
#0123456789.0 \
#0.0 \
#(-0.0) \
#(-1.0) \
#(-9876543210.0) \
#(-0987654321.0) \
#(-3.14) \
#(4.0 - 8.0)
