
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test styled operator.
$ bold(op("bold", limits: #true))_x y $
