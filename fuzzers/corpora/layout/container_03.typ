
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test fr box.
Hello #box(width: 1fr, rect(height: 0.7em, width: 100%)) World
