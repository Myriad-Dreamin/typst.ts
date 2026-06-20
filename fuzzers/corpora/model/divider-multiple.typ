
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test multiple dividers.
#set page(width: 200pt)
Section 1
#divider()
Section 2
#divider()
Section 3