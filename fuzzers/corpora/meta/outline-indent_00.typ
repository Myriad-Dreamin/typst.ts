
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// With heading numbering
#set page(width: 200pt)
#set heading(numbering: "1.a.")
#outline()
#outline(indent: false)
#outline(indent: true)
#outline(indent: none)
#outline(indent: auto)
#outline(indent: 2em)
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
