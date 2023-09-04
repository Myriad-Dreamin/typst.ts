
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Basic.
_Emphasized and *strong* words!_

// Inside of a word it's a normal underscore or star.
hello_world Nutzer*innen

// Can contain paragraph in nested content block.
_Still #[

] emphasized._
