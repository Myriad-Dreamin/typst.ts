
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test space between inner alignment points.
$ a + b &= 2 + 3 &= 5 \
      b &= c     &= 3 $
