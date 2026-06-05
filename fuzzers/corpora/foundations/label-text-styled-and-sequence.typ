
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test label on text, styled, and sequence.
#test([Hello<hi>].label, <hi>)
#test([#[A *B* C]<hi>].label, <hi>)
#test([#text(red)[Hello]<hi>].label, <hi>)