
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test that padding adding up to 100% does not panic.
#pad(50%)[]
