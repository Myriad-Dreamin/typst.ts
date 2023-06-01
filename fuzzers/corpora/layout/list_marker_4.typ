// Test list marker configuration.

#set page(width: 120pt, height: auto, margin: 10pt)

// Test that bare hyphen doesn't lead to cycles and crashes.
#set list(marker: [-])
- Bare hyphen is
- a bad marker
