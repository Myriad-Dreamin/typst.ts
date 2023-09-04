
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test alternative delimiter with set rule.
#set math.mat(delim: "[")
$ mat(1, 2; 3, 4) $
$ a + mat(delim: #none, 1, 2; 3, 4) + b $
