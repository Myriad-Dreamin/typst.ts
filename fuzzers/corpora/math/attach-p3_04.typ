
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test default of limit attachments on relations at all sizes
#set page(width: auto)
$ a =^"def" b quad a lt.eq_"really" b quad  a arrow.r.long.squiggly^"slowly" b $
$a =^"def" b quad a lt.eq_"really" b quad a arrow.r.long.squiggly^"slowly" b$

$a scripts(=)^"def" b quad a scripts(lt.eq)_"really" b quad a scripts(arrow.r.long.squiggly)^"slowly" b$
