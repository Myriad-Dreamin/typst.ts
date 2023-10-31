
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Specifying cancel line angle with an absolute angle
$cancel(x, angle: #0deg) + cancel(x, angle: #45deg) + cancel(x, angle: #90deg) + cancel(x, angle: #135deg)$
