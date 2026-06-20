
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// In this example, the "high-level" state introspection yielded the same
// value in iteration 4 and 5, but the "low-level" state query yielded a
// different sequence. It also converged, but we don't know that until one
// iteration later.
#import "switch.typ": switch
#let s = state("a", none)
#switch(n => if n == 5 { s.update(none) })
#context s.get()