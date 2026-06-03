
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
$ mat(;,) $
// Add some whitespace/trivia:
$ mat(; ,) $
$ mat(;/**/,) $
$ mat(;
,) $
$ mat(;// line comment
,) $
$ mat(
  1, , ;
   ,1, ;
   , ,1;
) $