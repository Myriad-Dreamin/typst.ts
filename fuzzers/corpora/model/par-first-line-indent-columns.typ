
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set par(first-line-indent: (amount: 1em, all: false))

A \ B

C \ D

#colbreak()

// No first line indent after column break
E \ F

G \ H