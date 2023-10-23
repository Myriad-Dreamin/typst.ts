
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test spacing for operators with decorations and modifiers on them
#set page(width: auto)
$a equiv b + c - d => e log 5 op("ln") 6$ \
$a cancel(equiv) b overline(+) c arrow(-) d hat(=>) e cancel(log) 5 dot(op("ln")) 6$ \
$a overbrace(equiv) b underline(+) c grave(-) d underbracket(=>) e circle(log) 5 caron(op("ln")) 6$ \
\
$a attach(equiv, tl: a, tr: b) b attach(limits(+), t: a, b: b) c tilde(-) d breve(=>) e attach(limits(log), t: a, b: b) 5 attach(op("ln"), tr: a, bl: b) 6$