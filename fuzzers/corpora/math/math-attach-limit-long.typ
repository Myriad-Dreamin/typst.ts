
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test long limit attachments.
$ attach(product, t: 123456789) attach(product, t: 123456789, bl: x) \
  attach(product, b: 123456789) attach(product, b: 123456789, tr: x) $
$attach(limits(product), t: 123456789) attach(limits(product), t: 123456789, bl: x)$

$attach(limits(product), b: 123456789) attach(limits(product), b: 123456789, tr: x)$