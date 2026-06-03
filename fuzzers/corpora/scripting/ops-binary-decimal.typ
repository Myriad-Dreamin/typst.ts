
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Addition.
#test(decimal("40.1") + decimal("13.2"), decimal("53.3"))
#test(decimal("12.34330") + decimal("45.96670"), decimal("58.31000"))
#test(decimal("451113.111111111111111111111") + decimal("23.222222222222222222324"), decimal("451136.333333333333333333435"))

// Subtraction.
#test(decimal("40.1") - decimal("13.2"), decimal("26.9"))
#test(decimal("12.34330") - decimal("45.96670"), decimal("-33.62340"))
#test(decimal("1234.111111111111111111111") - decimal("0.222222222222222222324"), decimal("1233.888888888888888888787"))

// Multiplication.
#test(decimal("40.5") * decimal("9.5"), decimal("384.75"))
#test(decimal("-0.1234567890123456789012345678") * decimal("-2.0"), decimal("0.2469135780246913578024691356"))

// Division.
#test(decimal("1.0") / decimal("7.0"), decimal("0.1428571428571428571428571429"))
#test(decimal("9999991.6666") / decimal("3.0"), decimal("3333330.5555333333333333333333"))
#test(decimal("3253452.4034029359598214312040") / decimal("-49293591.4039493929532"), decimal("-0.0660015290170614346071165643"))