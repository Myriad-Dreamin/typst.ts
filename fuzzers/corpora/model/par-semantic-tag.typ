
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#show par: highlight
#block[
  #metadata(none) <hi1>
  A
  #metadata(none) <hi2>
]

#block(width: 100%, metadata(none) + align(center)[A])
#block(width: 100%, align(center)[A] + metadata(none))