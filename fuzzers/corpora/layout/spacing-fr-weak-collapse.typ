
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Fractional weak spacing should collapse like rel weak spacing.
#set page(height: 100pt)
0
#v(1fr, weak: true)
#v(1fr, weak: false)
1
#v(1fr, weak: false)