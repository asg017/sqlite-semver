.load dist/debug/semver0
.mode box
.header on

create table tests as
  select column1
  from (
    values
    /*
    ('1.10.2'),
    ('1.1.2'),
    ('1.100.2')
    */
    ('1.0.0-alpha'),
    ('1.0.0-alpha.1'),
    ('1.0.0-alpha.beta'),
    ('v1.0.0-beta'),
    ('1.0.0-beta.2'),
    ('1.0.0-beta.11'),
    ('1.0.0-rc.1'),
    ('1.0.0')
  );

select * from tests;
select column1  as not_collated
from tests
order by column1;

select column1  as collated_asc
from tests
order by column1 collate semver asc;

select column1  as collated_desc
from tests
order by column1 collate semver desc;

select rowid, *, requirement from semver_requirements('>=1.2.3, <1.8');
.exit

select semver_matches('1.3.0', '>=1.2.3, <1.8.0');
select semver_matches('1.8.0', '>=1.2.3, <1.8.0');

select semver_version(1, 2, 3);
select semver_version(1, 2, 3, 'beta.3');
select semver_version(1, 2, 3, 'beta.3', 'zstd.1.5.0');
select semver_version(1, 2, 3, null, 'zstd.1.5.0');

select semver_version(1, 2, -3); -- TODO

select semver_gt('1.2.3', '1.2.2');
select semver_gt('1.2.2', '1.2.3');



