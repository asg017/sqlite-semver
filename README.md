A SQLite extension for parsing, sorting, and working with [Semantic Versioning](https://semver.org/) strings. Based on the [semver crate](https://crates.io/crates/semver) and [`sqlite-loadable-rs`](https://github.com/asg017/sqlite-loadable-rs).

Still a work in progress, not meant to be widely shared. But feel free to check it out! Will eventually be a part of the [`sqlite-ecosystem`](https://github.com/asg017/sqlite-ecosystem).

```sql
select semver_matches('1.3.0', '>=1.2.3, <1.8.0');
1

select semver_matches('1.8.0', '>=1.2.3, <1.8.0');
0


select * from semver_requirements('>=1.2.3, <1.8, ^1.99.0-alpha.2');
/*
┌────────────┬───────┬───────┬───────┬─────────┐
│     op     │ major │ minor │ patch │   pre   │
├────────────┼───────┼───────┼───────┼─────────┤
│ greater_eq │ 1     │ 2     │ 3     │         │
│ less       │ 1     │ 8     │       │         │
│ caret      │ 1     │ 99    │ 0     │ alpha.2 │
└────────────┴───────┴───────┴───────┴─────────┘
*/


select semver_version(1, 2, 3);
'1.2.3'

select semver_version(1, 2, 3, 'beta.3');
'1.2.3-beta.3'

select semver_version(1, 2, 3, 'beta.3', 'zstd.1.5.0');
'1.2.3-beta.3+zstd.1.5.0'
```

There's also a custom `"semver"` collation registered as part of the extension, so you can sort semver strings by their values.

```sql
create table tests as
  select column1 as version
  from (
    values
    ('1.0.0-rc.1'),
    ('1.0.0-beta.11'),
    ('1.0.0-alpha'),
    ('1.0.0-alpha.beta'),
    ('1.0.0'),
    ('v1.0.0-beta'),
    ('1.0.0-alpha.1'),
    ('1.0.0-beta.2')
  );

select version
from tests
order by version collate semver asc;
/*
┌──────────────────┐
│     version      │
├──────────────────┤
│ 1.0.0-alpha      │
│ 1.0.0-alpha.1    │
│ 1.0.0-alpha.beta │
│ v1.0.0-beta      │
│ 1.0.0-beta.2     │
│ 1.0.0-beta.11    │
│ 1.0.0-rc.1       │
│ 1.0.0            │
└──────────────────┘
*/
```
