
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Alignment shouldn't affect number
#set align(horizon)

+ ABCDEF\ GHIJKL\ MNOPQR
   + INNER\ INNER\ INNER
+ BACK\ HERE
