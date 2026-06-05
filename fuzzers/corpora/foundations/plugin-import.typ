
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#import plugin("/assets/plugins/hello.wasm"): hello, double_it

#test(hello(), bytes("Hello from wasm!!!"))
#test(double_it(bytes("hey!")), bytes("hey!.hey!"))