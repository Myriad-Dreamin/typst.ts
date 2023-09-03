
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test duplicate footnotes.
#let lang = footnote[Languages.]
#let nums = footnote[Numbers.]

/ "Hello": A word #lang
/ "123": A number #nums

- "Hello" #lang
- "123" #nums

+ "Hello" #lang
+ "123" #nums

#table(
  columns: 2,
  [Hello], [A word #lang],
  [123], [A number #nums],
)
