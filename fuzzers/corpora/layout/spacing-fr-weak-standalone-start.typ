
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Standalone fractional weak spacing should collapse.
#set page(height: 60pt)
#v(1fr, weak: true)
0