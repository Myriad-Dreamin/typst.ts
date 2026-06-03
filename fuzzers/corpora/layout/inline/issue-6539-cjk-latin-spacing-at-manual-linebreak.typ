
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Issue #6539
#set text(cjk-latin-spacing: auto)
#set box(width: 2.3em, stroke: (x: green))

#box(align(end)[甲国\ T国])

#box(align(end)[乙国 \ T国])

#box(align(end)[丙国 T国])

#box(align(end)[丁国T国])