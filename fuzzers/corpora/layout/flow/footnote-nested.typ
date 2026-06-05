
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
First \
Second #footnote[A, #footnote[B, #footnote[C]]]
Third #footnote[D, #footnote[E]] \
Fourth #footnote[F]