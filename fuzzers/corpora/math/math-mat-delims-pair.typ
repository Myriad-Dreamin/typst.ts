
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
$ mat(delim: #(none, "["), 1, 2; 3, 4) $
$ mat(delim: #(sym.chevron.r, sym.bracket.stroked.r), 1, 2; 3, 4) $