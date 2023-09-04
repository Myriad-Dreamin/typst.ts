
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Count labels.
#let label = <heya>
#let count = counter(label).display()
#let elem(it) = [#box(it) #label]

#elem[hey, there!] #count \
#elem[more here!] #count
