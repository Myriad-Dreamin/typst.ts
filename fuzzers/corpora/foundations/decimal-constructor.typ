
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#test(decimal(10), decimal("10.0"))
#test(decimal("-7654.321"), decimal("-7654.321"))
#test(decimal("\u{2212}7654.321"), decimal("-7654.321"))
#test(decimal({ 3.141592653 }), decimal("3.141592653000000012752934707"))
#test(decimal({ -3.141592653 }), decimal("-3.141592653000000012752934707"))
#test(decimal(decimal(3)), decimal("3.0"))
#test(decimal(true), decimal("1.0"))
#test(type(decimal(10)), decimal)