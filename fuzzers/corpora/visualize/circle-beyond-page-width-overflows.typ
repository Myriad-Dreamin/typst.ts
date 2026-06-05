
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that sizing a circle beyond the page width correctly overflows the page.
#set page(height: 100pt)
#circle(width: 150%)