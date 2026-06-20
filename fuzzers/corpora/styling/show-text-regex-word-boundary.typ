
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test caseless match and word boundaries.
#show regex("(?i)\\bworld\\b"): [🌍]

Treeworld, the World of worlds, is a world.