
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test customizing line via set rule.
#set page(width: 200pt)
#show divider: set line(stroke: 2pt + red)
Before
#divider()
After