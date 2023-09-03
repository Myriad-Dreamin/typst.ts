
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Count figures.
#figure(numbering: "A", caption: [Four 'A's], kind: image, supplement: "Figure")[_AAAA!_]
#figure(numbering: none, caption: [Four 'B's], kind: image, supplement: "Figure")[_BBBB!_]
#figure(caption: [Four 'C's], kind: image, supplement: "Figure")[_CCCC!_]
#counter(figure.where(kind: image)).update(n => n + 3)
#figure(caption: [Four 'D's], kind: image, supplement: "Figure")[_DDDD!_]
