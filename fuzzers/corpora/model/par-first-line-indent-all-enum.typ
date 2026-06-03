
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#show enum.where(tight: false): set enum(spacing: 1.2em)
#set par(
  first-line-indent: (amount: 12pt, all: true),
  spacing: 5pt,
  leading: 5pt,
)

+ A #parbreak() B #line(length: 100%) C

+ D