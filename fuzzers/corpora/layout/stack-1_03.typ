
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test aligning things in RTL stack with align function & fr units.
#set page(width: 50pt, margin: 5pt)
#set block(spacing: 5pt)
#set text(8pt)
#stack(dir: rtl, 1fr, [A], 1fr, [B], [C])
#stack(dir: rtl,
  align(center, [A]),
  align(left, [B]),
  [C],
)
