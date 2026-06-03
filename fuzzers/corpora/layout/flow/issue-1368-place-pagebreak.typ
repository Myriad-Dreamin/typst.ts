
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test placing on an already full page.
// It shouldn't result in a page break.
#set page(height: 40pt)
#block(height: 100%)
#place(bottom + right)[Hello world]