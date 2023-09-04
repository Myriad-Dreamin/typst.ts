
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test mixing lines with and some without alignment points.
$ "abc" &= c \
   &= d + 1 \
   = x $
