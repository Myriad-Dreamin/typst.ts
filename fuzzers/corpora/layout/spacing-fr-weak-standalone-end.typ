
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Standalone factional weak spacing at the end should collapse..
#set page(height: 60pt)
#v(1fr, weak: false)
1
#v(1fr, weak: true)