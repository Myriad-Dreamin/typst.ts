
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Warning: 6-11 the name `embed` is deprecated, use `attach` instead
// Hint: 6-11 it will be removed in Typst 0.15.0
#pdf.embed("/assets/text/hello.txt")