
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Count headings.
#set heading(numbering: "1.a.")
#show heading: set text(10pt)
#counter(heading).step()

= Alpha
In #counter(heading).display()
== Beta

#set heading(numbering: none)
= Gamma
#heading(numbering: "I.")[Delta]

At Beta, it was #locate(loc => {
  let it = query(heading, loc).find(it => it.body == [Beta])
  numbering(it.numbering, ..counter(heading).at(it.location()))
})
