
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that replacements happen exactly once.
#show "A": [BB]
#show "B": [CC]
AA (8)