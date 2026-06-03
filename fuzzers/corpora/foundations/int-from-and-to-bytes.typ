
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test `int.from-bytes` and `int.to-bytes`.
#test(int.from-bytes(bytes(())), 0)
#test(int.from-bytes(bytes((1, 0, 0, 0, 0, 0, 0, 0)), endian: "little", signed: true), 1)
#test(int.from-bytes(bytes((1, 0, 0, 0, 0, 0, 0, 0)), endian: "big", signed: true), 72057594037927936)
#test(int.from-bytes(bytes((1, 0, 0, 0, 0, 0, 0, 0)), endian: "little", signed: false), 1)
#test(int.from-bytes(bytes((255,)), endian: "big", signed: true), -1)
#test(int.from-bytes(bytes((255,)), endian: "big", signed: false), 255)
#test(int.from-bytes((-1000).to-bytes(endian: "big", size: 5), endian: "big", signed: true), -1000)
#test(int.from-bytes((-1000).to-bytes(endian: "little", size: 5), endian: "little", signed: true), -1000)
#test(int.from-bytes(1000.to-bytes(endian: "big", size: 5), endian: "big", signed: true), 1000)
#test(int.from-bytes(1000.to-bytes(endian: "little", size: 5), endian: "little", signed: true), 1000)
#test(int.from-bytes(1000.to-bytes(endian: "little", size: 5), endian: "little", signed: false), 1000)