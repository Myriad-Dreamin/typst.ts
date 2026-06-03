
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let rotations = range(1, 9)
#let with-rotation(path, offset, v) = {
  let data = read(path, encoding: none)
  let modified = data.slice(0, offset) + bytes((v,)) + data.slice(offset + 1)
  image(modified, width: 10pt)
}

#set page(width: auto)
#table(
  columns: 1 + rotations.len(),
  table.header(
    [], ..rotations.map(v => raw(str(v), lang: "typc")),
  ),
  `PNG`, ..rotations.map(v => with-rotation("/assets/images/f2t.png", 0x85, v)),
  // JPEG has special handing in PDF export (no recoding, so instead we use a
  // transform to apply the orientation), so it's worth testing that separately.
  `JPEG`, ..rotations.map(v => with-rotation("/assets/images/f2t.jpg", 0x31, v)),
)