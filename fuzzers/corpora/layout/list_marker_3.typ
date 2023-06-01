// Test list marker configuration.

#set page(width: 120pt, height: auto, margin: 10pt)

// Test function.
#set list(marker: n => if n == 1 [--] else [•])
- A
- B
  - C
  - D
    - E
- F