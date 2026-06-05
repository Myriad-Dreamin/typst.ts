
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Text inside raw block should follow the specified alignment.
#set page(width: 180pt)
#set text(6pt)

#align(center, raw(
  lang: "typ",
  block: true,
  align: right,
  "#let f(x) = x\n#align(center, line(length: 1em))",
))