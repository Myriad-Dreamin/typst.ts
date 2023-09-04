
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Linebreak and leading-sized weak spacing are equivalent.
#box[A \ B] #box[A #v(0.65em, weak: true) B]

// Eating up soft spacing.
Inv#h(0pt)isible

// Multiple spacings in a row.
Add #h(10pt) #h(10pt) up

// Relative to area.
#let x = 25% - 4pt
|#h(x)|#h(x)|#h(x)|#h(x)|

// Fractional.
| #h(1fr) | #h(2fr) | #h(1fr) |
