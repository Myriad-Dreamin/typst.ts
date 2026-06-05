
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Big numbers (larger than what i64 can store) should just lose some precision
// but not overflow
#let bignum = json("/assets/data/big-number.json")
#bignum