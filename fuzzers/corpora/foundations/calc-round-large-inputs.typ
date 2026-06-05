
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#test(calc.round(31114, digits: 4000000000), 31114)
#test(calc.round(9223372036854775807, digits: 12), 9223372036854775807)
#test(calc.round(9223372036854775807, digits: -20), 0)
#test(calc.round(238959235.129590203, digits: 4000000000), 238959235.129590203)
#test(calc.round(1.7976931348623157e+308, digits: 12), 1.7976931348623157e+308)
#test(calc.round(1.7976931348623157e+308, digits: -308), float.inf)
#test(calc.round(-1.7976931348623157e+308, digits: -308), -float.inf)
#test(calc.round(12.34, digits: -312), 0.0)
#test(calc.round(decimal("238959235.129590203"), digits: 4000000000), decimal("238959235.129590203"))
#test(calc.round(decimal("79228162514264337593543950335"), digits: 12), decimal("79228162514264337593543950335"))
#test(calc.round(decimal("79228162514264337593543950335"), digits: -50), decimal("0"))
#test(calc.round(decimal("-79228162514264337593543950335"), digits: -2), decimal("-79228162514264337593543950300"))