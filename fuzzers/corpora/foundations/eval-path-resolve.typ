
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test absolute path.
#eval("image(\"/assets/images/tiger.jpg\", width: 50%)")