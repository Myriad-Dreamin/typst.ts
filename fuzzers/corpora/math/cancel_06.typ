
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Specifying cancel line angle with a function
$x + cancel(y, angle: #{angle => angle + 90deg}) - cancel(z, angle: #(angle => angle + 135deg))$
$ e + cancel((j + e)/(f + e)) - cancel((j + e)/(f + e), angle: #(angle => angle + 30deg)) $
