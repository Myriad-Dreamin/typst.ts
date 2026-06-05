
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Warning: 40-49 linebreaks are ignored in branches
// Hint: 40-49 use commas instead to separate each line
$ cases(a, b, c) cases(reverse: #true, a \ b \ c) $