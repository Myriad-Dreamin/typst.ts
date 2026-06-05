
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#test(float(10), 10.0)
#test(float(50% * 30%), 0.15)
#test(float("31.4e-1"), 3.14)
#test(float("31.4e\u{2212}1"), 3.14)
#test(float("3.1415"), 3.1415)
#test(float("-7654.321"), -7654.321)
#test(float("\u{2212}7654.321"), -7654.321)
#test(float(decimal("4.89")), 4.89)
#test(float(decimal("3.1234567891234567891234567891")), 3.123456789123457)
#test(float(decimal("79228162514264337593543950335")), 79228162514264340000000000000.0)
#test(float(decimal("-79228162514264337593543950335")), -79228162514264340000000000000.0)
#test(type(float(10)), float)