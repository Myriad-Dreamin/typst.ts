
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let check(expected) = context assert.eq(par.first-line-indent, expected)

// To be intuitive, values from context should never contain `none`.
#check((amount: 0pt, all: false))

#set par(first-line-indent: 2em)
#check((amount: 2em, all: false))

#set par(first-line-indent: (all: true))
#check((amount: 2em, all: true))

/// The following two ways should be the same.
#set par(first-line-indent: 7em)
#check((amount: 7em, all: true))
#set par(first-line-indent: (amount: 1em))
#check((amount: 1em, all: true))

#set par(first-line-indent: (all: false))
#check((amount: 1em, all: false))

#set par(first-line-indent: (amount: 8em, all: true))
#check((amount: 8em, all: true))