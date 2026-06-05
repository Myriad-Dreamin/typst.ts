
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that `mid` creates a Relation, but that can be overridden.
$ (a | b) $
$ (a mid(|) b) $
$ (a class("unary", |) b) $
$ (a class("unary", mid(|)) b) $
$ (a mid(class("unary", |)) b) $