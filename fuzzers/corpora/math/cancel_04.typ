
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Resized and styled
#set page(width: 200pt, height: auto)
$a + cancel(x, length: #200%) - cancel(x, length: #50%, stroke: #(red + 1.1pt))$
$ b + cancel(x, length: #150%) - cancel(a + b + c, length: #50%, stroke: #(blue + 1.2pt)) $
