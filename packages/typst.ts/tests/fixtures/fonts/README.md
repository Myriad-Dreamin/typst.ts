# WOFF font fixtures

`LibertinusSerif-Regular-subset.woff` and `LibertinusSerif-Regular-subset.woff2`
are WOFF / WOFF2 transcodings of `assets/data/LibertinusSerif-Regular-subset.otf`
(an SIL OFL licensed font, checked in for typst's own data tests), produced with
[fontTools](https://fonttools.readthedocs.io/) so the fixtures come from an
implementation independent of the one under test.

To regenerate, from the repository root:

```sh
python - <<'EOF'
from fontTools.ttLib import TTFont
src = 'assets/data/LibertinusSerif-Regular-subset.otf'
for flavor, ext in [('woff', '.woff'), ('woff2', '.woff2')]:
    font = TTFont(src)
    font.flavor = flavor
    font.save('packages/typst.ts/tests/fixtures/fonts/LibertinusSerif-Regular-subset' + ext)
EOF
```

Note: fontTools normalizes the `head` table (`checkSumAdjustment`, the
`modified` timestamp, and the glyph bounding box) and may reorder the physical
table layout when transcoding, so the fixtures are not byte-for-byte copies of
the OTF. The round-trip test in `src/woff.node.test.mts` compares per-table
contents accordingly.
