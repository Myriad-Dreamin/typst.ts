
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// If the semicolon directly follows a hash expression, it terminates that
// instead of indicating 2d arguments.
$ mat(#"math" ; "wins") $
$ mat(#"code"; "wins") $