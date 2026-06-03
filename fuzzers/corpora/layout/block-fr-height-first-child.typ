
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that block spacing is not trimmed if only an fr block precedes it.
#set page(height: 100pt)
#rect(height: 1fr)
#rect()