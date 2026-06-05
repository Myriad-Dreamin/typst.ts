
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that delimiters have opening and closing math class.
$ 2mat(a, delim: bar.v) 2 $
$ 2 mat(a, delim: bar.v)2 $