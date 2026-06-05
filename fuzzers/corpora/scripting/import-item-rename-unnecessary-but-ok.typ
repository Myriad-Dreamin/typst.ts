
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#import "modul" + "e.typ" as module
#test(module.b, 1)