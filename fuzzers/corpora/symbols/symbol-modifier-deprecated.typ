
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Warning: 7-12 `ast.small` is deprecated (CJK compatibility character), use ﹡ or `\u{fe61}` instead
$ ast.small $

// Warning: 14-20 `bracket.double` is deprecated, use `bracket.stroked` instead
#sym.bracket.double.r