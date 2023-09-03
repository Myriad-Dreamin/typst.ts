
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Multiple footnotes are refs
First #footnote[A]<fn1> \
Second #footnote[B]<fn2> \
First ref @fn1 \
Third #footnote[C] \
Fourth #footnote[D]<fn4> \
Fourth ref @fn4 \
Second ref @fn2 \
Second ref again @fn2
