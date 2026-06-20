
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Verify whether overriding vertical alignment causes horizontal alignment to
// be inherited from the context.
#set align(center)
#set enum(
  number-align: top,
  numbering: n => "1" * n,
)

+ abc
+ abc
+ abc