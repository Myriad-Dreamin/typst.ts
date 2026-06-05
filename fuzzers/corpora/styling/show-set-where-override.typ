
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#show heading: set text(green)
#show heading.where(level: 1): set text(red)
#show heading.where(level: 2): set text(blue)
= Red
== Blue
=== Green