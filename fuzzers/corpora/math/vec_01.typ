
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test alternative delimiter.
#set math.vec(delim: "[")
$ vec(1, 2) $
