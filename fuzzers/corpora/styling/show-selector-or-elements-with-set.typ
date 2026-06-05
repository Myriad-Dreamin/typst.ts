
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Looking forward to `heading.where(level: 1 | 2)` :)
#show heading.where(level: 1).or(heading.where(level: 2)): set text(red)
= L1
== L2
=== L3
==== L4