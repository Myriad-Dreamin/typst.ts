
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page


#set page(
  paper: "a7",
  numbering: "1 / 1",
  margin: (bottom: 1cm, rest: 0.5cm),
)

#set heading(outlined: true, numbering: "1.")

// This is purposefully an empty
#locate(loc => [
  Non-outlined elements:
  #(query(selector(heading).and(heading.where(outlined: false)), loc)
    .map(it => it.body).join(", "))
])

#heading("A", outlined: false)
#heading("B", outlined: true)
#heading("C", outlined: true)
#heading("D", outlined: false)
