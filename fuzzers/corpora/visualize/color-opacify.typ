
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#test-repr(luma(100%, 50%).opacify(50%), luma(100%, 75%))
#test-repr(luma(100%, 20%).opacify(100%), luma(100%, 100%))
#test-repr(luma(100%, 100%).opacify(250%), luma(100%, 100%))
#test-repr(luma(100%, 50%).opacify(-50%), luma(100%, 25%))
#test-repr(luma(100%, 0%).opacify(0%), luma(100%, 0%))