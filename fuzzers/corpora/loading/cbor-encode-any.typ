
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#import "edge-case.typ": special-types
#for value in special-types {
  test(
    cbor.encode(value),
    cbor.encode(repr(value)),
  )
}