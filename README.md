> [!NOTE]
> This project is currently on hold, as Gothic modding community seems to be a lot more dead than I initially anticipated, so I'm not sure if I want to spend my time working on tooling that barely anyone needs.

# DaedalusKit

Modern tooling for Daedalus language.

Daedalus was used to develop cult classic like [Gothic](https://en.wikipedia.org/wiki/Gothic_(video_game)) and [Gothic 2](https://en.wikipedia.org/wiki/Gothic_(video_game)) releassed in early 2000s and more recently in [Chronicles Of Myrtana](https://kronikimyrtany.pl/en).

### Project components (✅ = Done 🚧 = WIP)

- `daedalus-lexer` - Lexer for the language ✅ 
- `daedalus-parser` - Parser for the language ✅ 
- `output-units` - ✅
  - Uses the lexer to generates in-game dialog prompts file
  - Produces output byte compatible with the original engine, but orders of magnitude faster
- `daedalus-fmt` - Opinionated language formatter 🚧
- `daedalus-compiler` - Compiles the code ✅ (For now tested only against G2 Notr)
  - `daedalus-bytecode` - Representation of the bytecode format ✅
  - `dat-file` - Implementation of the file format used to store the bytecode and symbol definitions ✅
- `interner` - String interner with support for case insetive interning needed for Daedalus
- `daedalus-lsp` - Modern language server (not even started, only planned so far)
