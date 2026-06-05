
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Try same key with different initial value.
#context state("key", 2).get()
#state("key").update(x => x + 1)
#context state("key", 2).get()
#context state("key", 3).get()
#state("key").update(x => x + 1)
#context state("key", 2).get()