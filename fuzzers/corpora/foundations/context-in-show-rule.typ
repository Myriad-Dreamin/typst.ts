
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that show rule establishes context.
#set heading(numbering: "1.")
#show heading: it => test(
  counter(heading).get(),
  (intro: (1,), back: (2,)).at(str(it.label)),
)

= Introduction <intro>
= Background <back>