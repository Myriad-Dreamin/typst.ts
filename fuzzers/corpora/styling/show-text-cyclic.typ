
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test direct cycle.
#show "Hello": text(red)[Hello]
Hello World!