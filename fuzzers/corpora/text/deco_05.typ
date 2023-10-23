
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test a tighter highlight.
#set highlight(top-edge: "x-height", bottom-edge: "baseline")
#highlight[ace],
#highlight[base],
#highlight[super],
#highlight[phone #sym.integral]
