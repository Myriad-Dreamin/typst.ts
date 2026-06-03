
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#show terms.where(tight: false): set terms(spacing: 1.2em)
#set terms(hanging-indent: 10pt)
#set par(
  first-line-indent: (amount: 12pt, all: true),
  spacing: 5pt,
  leading: 5pt,
)

/ Term A: B \ C #parbreak() D #line(length: 100%) E

/ Term F: G