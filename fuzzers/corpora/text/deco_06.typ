
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test a bounds highlight.
#set highlight(top-edge: "bounds", bottom-edge: "bounds")
#highlight[abc]
#highlight[abc #sym.integral]
