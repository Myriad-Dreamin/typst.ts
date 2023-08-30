// Bytefield - generate protocol headers and more
// Feel free to contribute with any features you think are missing.
// Still a WIP - alpha stage and a bit hacky at the moment

#import "@preview/tablex:0.0.4": tablex, cellx, gridx
#set text(font: "IBM Plex Mono")

#let bfcell(
  len, // lenght of the fields in bits 
  content, 
  fill: none, // color to fill the field
  height: auto // height of the field
) = cellx(colspan: len, fill: fill, inset: 0pt)[#box(height: height, width: 100%, stroke: 1pt + black)[#content]]


#let bytefield(bits: 32, rowheight: 2.5em, bitheader: auto, ..fields) = {
  // state variables
  let col_count = 0
  let cells = ()

  // calculate cells
  for (idx, field) in fields.pos().enumerate() {
    let (size, content, fill, ..) = field;
    let remaining_cols = bits - col_count;
    col_count = calc.rem(col_count + size, bits);
    // if no size was specified
    if size == none {
      size = remaining_cols
      content = content + sym.star
    }
    if size > bits and remaining_cols == bits and calc.rem(size, bits) == 0 {
      content = content + " (" + str(size) + " Bit)"
      cells.push(bfcell(int(bits),fill:fill, height: rowheight * size/bits)[#content])
      size = 0
    }

    while size > 0 {
      let width = calc.min(size, remaining_cols);
      size -= remaining_cols
      remaining_cols = bits
      cells.push(bfcell(int(width),fill:fill, height: rowheight,)[#content])
    }
  
  }

  bitheader = if bitheader == auto { 
    range(bits).map(i => if calc.rem(i,8) == 0 or i == (bits - 1) { text(9pt)[#i] } else { none }) 
  } else if bitheader != none {
    assert(type(bitheader) == "array", message: "header must be an array or none")
    range(bits).map(i => if i in bitheader { text(9pt)[#i] } else {none})
  }
  
  box(width: 100%)[
    //#show grid: set block(below: 0pt)
    #gridx(
      columns: range(bits).map(i => 1fr),
      align: center + horizon,
      //inset:0pt,
      ..bitheader,
      ..cells,
    )
  ]
}

// Low level API
#let bitbox(length_in_bits, content, fill: none) = (
  type: "bitbox",
  size: length_in_bits,   // length of the field 
  fill: fill,
  content: content,
  var: false, 
  show_size: false,
)

// High level API
#let bit(..args) = bitbox(1, ..args)
#let bits(len, ..args) = bitbox(len, ..args)
#let byte(..args) = bitbox(8, ..args)
#let bytes(len, ..args) = bitbox(len * 8, ..args)
#let padding(..args) = bitbox(none, ..args)

// Rotating text for flags
#let flagtext(text) = align(end,pad(-3pt,rotate(270deg,text)))

// Common network protocols
#let ipv4 = bytefield(
  bits(4)[Version], bits(4)[TTL], bytes(1)[TOS], bytes(2)[Total Length],
  bytes(2)[Identification], bits(3)[Flags], bits(13)[Fragment Offset],
  bytes(1)[TTL], bytes(1)[Protocol], bytes(2)[Header Checksum],
  bytes(4)[Source Address],
  bytes(4)[Destination Address],
  bytes(3)[Options], bytes(1)[Padding]
)

#let ipv6 = bytefield(
  bits(4)[Version], bytes(1)[Traffic Class], bits(20)[Flowlabel],
  bytes(2)[Payload Length], bytes(1)[Next Header], bytes(1)[Hop Limit],
  bytes(128/8)[Source Address],
  bytes(128/8)[Destination Address],
)

#let icmp = bytefield(
  header: (0,8,16,31),
  byte[Type], byte[Code], bytes(2)[Checksum],
  bytes(2)[Identifier], bytes(2)[Sequence Number],
  padding[Optional Data ]
)

#let icmpv6 = bytefield(
  header: (0,8,16,31),
  byte[Type], byte[Code], bytes(2)[Checksum],
  padding[Internet Header + 64 bits of Original Data Datagram  ]
)

#let dns = bytefield(
  bytes(2)[Identification], bytes(2)[Flags],
  bytes(2)[Number of Questions], bytes(2)[Number of answer RRs],
  bytes(2)[Number of authority RRs], bytes(2)[Number of additional RRs],
  bytes(8)[Questions],
  bytes(8)[Answers (variable number of resource records) ],
  bytes(8)[Authority (variable number of resource records) ],
  bytes(8)[Additional information (variable number of resource records) ],
)

#let tcp = bytefield(
  bytes(2)[Source Port], bytes(2)[ Destinatino Port],
  bytes(4)[Sequence Number],
  bytes(4)[Acknowledgment Number],
  bits(4)[Data Offset],bits(6)[Reserved], bits(6)[Flags], bytes(2)[Window],
  bytes(2)[Checksum], bytes(2)[Urgent Pointer],
  bytes(3)[Options], byte[Padding],
  padding[...DATA...]
)



#let tcp_detailed = bytefield(
  bytes(2)[Source Port], bytes(2)[ Destinatino Port],
  bytes(4)[Sequence Number],
  bytes(4)[Acknowledgment Number],
  bits(4)[Data Offset],bits(6)[Reserved], bit[#flagtext("URG")], bit[#flagtext("ACK")], bit[#flagtext("PSH")], bit[#flagtext("RST")], bit[#flagtext("SYN")], bit[#flagtext("FIN")], bytes(2)[Window],
  bytes(2)[Checksum], bytes(2)[Urgent Pointer],
  bytes(3)[Options], byte[Padding],
  padding[...DATA...]
)

#let udp = bytefield(
  bitheader: (0,16,31),
  bytes(2)[Source Port], bytes(2)[ Destinatino Port],
  bytes(2)[Length], bytes(2)[Checksum],
  padding[...DATA...]
)
