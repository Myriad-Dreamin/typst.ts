
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Same thing, but even more cursed, because `kind` is synthesized.
#show figure.where(kind: table): set figure(kind: raw)
#figure(table[A], caption: [Code])