
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Try measuring a citation that appears inline with other stuff. The
// introspection-assisted location assignment will ensure that the citation
// in the measurement is matched up with the real one.
#context {
  let it = [@netwok]
  let size = measure(it)
  place(line(length: size.width))
  v(1mm)
  it + [ is cited]
}

#show bibliography: none
#bibliography("/assets/bib/works.bib")