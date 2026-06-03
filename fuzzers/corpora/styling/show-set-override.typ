
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test overriding show-set rules.
#show strong: set text(red)
Hello *World*

#show strong: set text(blue)
Hello *World*