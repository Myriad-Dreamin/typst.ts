
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Without heading numbering
#set page(width: 200pt)
#outline()
#outline(indent: false)
#outline(indent: true)
#outline(indent: none)
#outline(indent: auto)
#outline(indent: n => 2em * n)
#outline(indent: n => ([-], [], [==], [====]).at(n))
#outline(indent: n => "!" * calc.pow(2, n))

= About ACME Corp.

== History
#lorem(10)

== Products
#lorem(10)

=== Categories
#lorem(10)

==== General
#lorem(10)
