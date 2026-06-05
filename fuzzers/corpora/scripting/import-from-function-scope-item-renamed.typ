
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test renaming items imported from function scopes.
#import assert: eq as aseq
#aseq(10, 10)