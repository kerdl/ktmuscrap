# Structure
- `css`: css parsing
- `group`: group identifier parsing and validation
- `sheet`: where the main table parsing happens
- `subject`: subject string parsing, extracting
subject name, attenders and their cabinets
- `teacher`: teacher identifier parsing and validation
- `attender`: teacher as an attender parsing,
cabinet extraction
- `cabinet`: extracting cabinet from teacher schedules
- `date`: date parsing
- `mod.rs`: parsing entry point
- `node.rs`: tools to work with HTML nodes