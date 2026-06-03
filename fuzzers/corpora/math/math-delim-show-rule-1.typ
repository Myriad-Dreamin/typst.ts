
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#show regex("\[|\]"): set text(green, font: "Noto Sans Math")
$ mat(delim: \[, a, b, c; d, e, f; g, h, i) quad [x + y] $