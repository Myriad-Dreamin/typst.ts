
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let value = bytes("Typst")
#test(cbor(cbor.encode(value)), value)