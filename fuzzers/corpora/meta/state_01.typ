
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Try same key with different initial value.
#state("key", 2).display()
#state("key").update(x => x + 1)
#state("key", 2).display()
#state("key", 3).display()
#state("key").update(x => x + 1)
#state("key", 2).display()
