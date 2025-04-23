
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Count headings.
#set heading(numbering: "1.a.")
#show heading: set text(10pt)
#counter(heading).step()

= Alpha
In #context counter(heading).display()
== Beta

#set heading(numbering: none)
= Gamma
#heading(numbering: "I.")[Delta]

At Beta, it was #context {
  let it = query(heading).find(it => it.body == [Beta])
  numbering(it.numbering, ..counter(heading).at(it.location()))
}
