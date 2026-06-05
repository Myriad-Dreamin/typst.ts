
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test if the math class changes the limit configuration.
$ class("normal", ->)_a $
$class("relation", x)_a$
$ class("large", x)_a $
$class("large", ->)_a$

$limits(class("normal", ->))_a$
$ scripts(class("relation", x))_a $