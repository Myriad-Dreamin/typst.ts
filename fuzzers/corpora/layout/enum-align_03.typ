
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Number align option should not be affected by the context.
#set align(center)
#set enum(number-align: start)

4.  c
8.  d
16. e\ f
   2.  f\ g
   32. g
   64. h
