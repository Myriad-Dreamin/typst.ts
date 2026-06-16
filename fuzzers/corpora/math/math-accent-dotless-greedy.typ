
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Currently the dotless style propagates to everything in the accent's base,
// even though it shouldn't.
$ arrow(P_(c, i dot j) P_(1, i) j) \
  arrow(P_(c, i dot j) P_(1, i) j, dotless: #false) $