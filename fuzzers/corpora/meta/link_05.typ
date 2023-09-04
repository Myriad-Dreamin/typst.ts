
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Styled with underline and color.
#show link: it => underline(text(fill: rgb("283663"), it))
You could also make the
#link("https://html5zombo.com/")[link look way more typical.]
