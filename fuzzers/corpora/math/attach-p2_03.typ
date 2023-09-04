
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#set text(size: 8pt)

// Test that the attachments are aligned horizontally.
$ x_1 p_1 frak(p)_1 2_1 dot_1 lg_1 !_1 \\_1 ]_1 "ip"_1 op("iq")_1 \
  x^1 b^1 frak(b)^1 2^1 dot^1 lg^1 !^1 \\^1 ]^1 "ib"^1 op("id")^1 \
  x_1 y_1 "_"_1 x^1 l^1 "`"^1 attach(I,tl:1,bl:1,tr:1,br:1)
  scripts(sum)_1^1 integral_1^1 |1/2|_1^1 \
  x^1_1, "("b y")"^1_1 != (b y)^1_1, "[∫]"_1 [integral]_1 $
