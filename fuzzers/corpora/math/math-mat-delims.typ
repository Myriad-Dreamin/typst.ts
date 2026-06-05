
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
$ mat(delim: #none, 1, 2; 3, 4) $

$ mat(delim: "(", 1, 2; 3, 4) $
$ mat(delim: \(, 1, 2; 3, 4) $
$ mat(delim: paren.l, 1, 2; 3, 4) $

$ mat(delim: "[", 1, 2; 3, 4) $
$ mat(delim: \[, 1, 2; 3, 4) $
$ mat(delim: bracket.l, 1, 2; 3, 4) $

$ mat(delim: "⟦", 1, 2; 3, 4) $
$ mat(delim: bracket.stroked.l, 1, 2; 3, 4) $

$ mat(delim: "{", 1, 2; 3, 4) $
$ mat(delim: \{, 1, 2; 3, 4) $
$ mat(delim: brace.l, 1, 2; 3, 4) $

$ mat(delim: "|", 1, 2; 3, 4) $
$ mat(delim: \|, 1, 2; 3, 4) $
$ mat(delim: bar.v, 1, 2; 3, 4) $

$ mat(delim: "‖", 1, 2; 3, 4) $
$ mat(delim: bar.v.double, 1, 2; 3, 4) $

$ mat(delim: "⟨", 1, 2; 3, 4) $
$ mat(delim: chevron.l, 1, 2; 3, 4) $