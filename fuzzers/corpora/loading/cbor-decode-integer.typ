
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#import "edge-case.typ": cbor-integers
#let data = cbor(cbor-integers)

#for (name, result) in data.representable {
  assert.eq(
    type(result),
    int,
    message: "failed to decode " + name,
  )
}

#for (name, result) in data.large {
  assert.eq(
    type(result),
    float,
    message: "failed to approximately decode " + name,
  )
}