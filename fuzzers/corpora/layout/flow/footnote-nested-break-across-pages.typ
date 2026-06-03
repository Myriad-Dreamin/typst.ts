
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 80pt)
A #footnote([I: ] + lines(6) + footnote[II])
B #footnote[III]