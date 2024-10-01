# Code structure tour
Some directories have a `README.md`
describing the purpose of
included files and folders.


## Update loop
Schedule download loop is located at [`crate::data::schedule::raw::Index::update_forever`](/src/data/schedule/raw/index.rs?blame=1#L281).

From there, after the download finishes, a signal is sent that is then received by 
[`crate::data::container::schedule::Schedule::await_updates`](/src/data/container/schedule.rs?blame=1#L77).

The schedules are then parsed, merged and compared there.


## Parsing
Happens in 2 steps:
- HTML → table ([`crate::parse::sheet::html`](/src/parse/sheet/html.rs))
- Table → objects ([`crate::parse::sheet::table`](/src/parse/sheet/table.rs))


### First step
Converts HTML to a 2D array.
- Empty strings are filtered out
- CSS is parsed to determine cell colors
- X and Y coordinates are calculated
to determine merged cells

In this table, these cell properties
are saved:
- Text
- Width and height
- Color (to determine subject format)


### Second step
Converts the table to final objects.
- Checking cells on the date row
- Recognizing merged cells
- Parsing cell contents
- Associating cells with dates and formations
using coordinates


## Merging
- Same schedule types are merged here: 
[`crate::merge::combine`](/src/merge/mod.rs?blame=1#L370).

- Complementing happens here: 
[`crate::merge::complement`](/src/merge/mod.rs?blame=1#L29).

  - Happens in two iterations, firsly teachers are 
    complemented from groups, and then groups are
    complemented from teachers

    Teachers are searched using the
    Damerau-Levenshtein distance.


## Comparing
There are a few comparing models used:
- Primitive ([`crate::compare::Primitive`](/src/compare/mod.rs?blame=1#L227)), whre the value
is singular (subject number, cabinet).
Holds the old and a new value.
- Detailed ([`crate::compare::DetailedChanges`](/src/compare/mod.rs?blame=1#L88)),
where lists are used (list of formations,
subject attenders).
Holds appeared, disappeared and changed types.

Schedule objects implement a trait,
which is used to compare themselves with each other:
[`crate::compare::FindingCmp`](/src/compare/mod.rs?blame=1#L19).

"Comparing" versions of regular schedule objects
are located in [`crate::compare::schedule`](/src/compare/schedule.rs).


## Schedule objects
Schedule object are described in
[`crate::data::schedule`](/src/data/schedule/mod.rs).

- `Page`: contains formations
(either groups or teachers)
- `Formation`: Either a group or a teacher,
contains days
- `Day`: contains subjects
- `Subject`: contains attenders (either groups or teachers)
- `Attender`: Either a group or a teacher,
contains the cabinet
- `Cabinet`: contains a primary variant and
a variant found in the opposite schedule


## API
Response objects are located in [`crate::api`](/src/api/mod.rs).

Schedule endpoints are in
[`crate::api::schedule::groups`](/src/api/schedule/groups.rs) and
[`crate::api::schedule::teachers`](/src/api/schedule/teachers.rs).

WebSocket channel code is in
[`crate::api::schedule`](/src/api/schedule/mod.rs).