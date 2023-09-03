
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#set page(width: auto, height: auto)

// Test with auto-sized page.
First
#pagebreak(to: "odd")
Third
