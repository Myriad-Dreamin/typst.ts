
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test styling dotless i and j.
$ dotless.i dotless.j,
  upright(dotless.i) upright(dotless.j),
  sans(dotless.i) sans(dotless.j),
  bold(dotless.i) bold(dotless.j),
  bb(dotless.i) bb(dotless.j),
  cal(dotless.i) cal(dotless.j),
  frak(dotless.i) frak(dotless.j),
  mono(dotless.i) mono(dotless.j),
  bold(frak(dotless.i)) upright(sans(dotless.j)),
  italic(bb(dotless.i)) frak(sans(dotless.j)) $