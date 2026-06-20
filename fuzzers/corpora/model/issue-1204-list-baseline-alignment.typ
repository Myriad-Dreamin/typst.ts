
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
- A
- $ sum_(i = 1)^n overbrace(x^6, y) $
- #box(baseline: 1cm)[C]
- #v(1cm) D
- #text(48pt)[E]
- #block(inset: 10pt, stroke: red)[Hello world!]
- #rect[Hello world!]