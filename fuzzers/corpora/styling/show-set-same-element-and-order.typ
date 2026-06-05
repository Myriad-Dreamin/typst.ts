
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test both things at once.
#show heading: set text(red)
= Level 1
== Level 2

#show heading.where(level: 1): set text(blue)
#show heading.where(level: 1): set text(green)
#show heading.where(level: 1): set heading(numbering: "(I)")
= Level 1
== Level 2