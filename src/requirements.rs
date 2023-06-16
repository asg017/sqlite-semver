use semver::{Prerelease, VersionReq};
use sqlite_loadable::{
    api,
    table::{ConstraintOperator, IndexInfo, VTab, VTabArguments, VTabCursor},
    BestIndexError, Result,
};
use sqlite_loadable::{prelude::*, Error};

use std::{mem, os::raw::c_int};

static CREATE_SQL: &str =
    "CREATE TABLE x(op text, major int not null, minor int, patch int, pre text, requirement hidden)";
enum Columns {
    Op,
    Major,
    Minor,
    Patch,
    Pre,
    Requirement,
}
fn column(index: i32) -> Option<Columns> {
    match index {
        0 => Some(Columns::Op),
        1 => Some(Columns::Major),
        2 => Some(Columns::Minor),
        3 => Some(Columns::Patch),
        4 => Some(Columns::Pre),
        5 => Some(Columns::Requirement),
        _ => None,
    }
}

#[repr(C)]
pub struct SemverRequrementsTable {
    /// must be first
    base: sqlite3_vtab,
}

impl<'vtab> VTab<'vtab> for SemverRequrementsTable {
    type Aux = ();
    type Cursor = RegexFindAllCursor;

    fn connect(
        _db: *mut sqlite3,
        _aux: Option<&Self::Aux>,
        _args: VTabArguments,
    ) -> Result<(String, SemverRequrementsTable)> {
        let base: sqlite3_vtab = unsafe { mem::zeroed() };
        let vtab = SemverRequrementsTable { base };
        // TODO db.config(VTabConfig::Innocuous)?;
        Ok((CREATE_SQL.to_owned(), vtab))
    }
    fn destroy(&self) -> Result<()> {
        Ok(())
    }

    fn best_index(&self, mut info: IndexInfo) -> core::result::Result<(), BestIndexError> {
        let mut has_requirement = false;
        for mut constraint in info.constraints() {
            match column(constraint.column_idx()) {
                Some(Columns::Requirement) => {
                    if constraint.usable() && constraint.op() == Some(ConstraintOperator::EQ) {
                        constraint.set_omit(true);
                        constraint.set_argv_index(1);
                        has_requirement = true;
                    } else {
                        return Err(BestIndexError::Constraint);
                    }
                }
                _ => (),
            }
        }
        if !has_requirement {
            return Err(BestIndexError::Error);
        }
        info.set_estimated_cost(100000.0);
        info.set_estimated_rows(100000);
        info.set_idxnum(2);

        Ok(())
    }

    fn open(&mut self) -> Result<RegexFindAllCursor> {
        Ok(RegexFindAllCursor::new())
    }
}

#[repr(C)]
pub struct RegexFindAllCursor {
    /// Base class. Must be first
    base: sqlite3_vtab_cursor,
    requirement: String,
    version_requirement: Option<VersionReq>,
    current_idx: usize,
}
impl RegexFindAllCursor {
    fn new() -> RegexFindAllCursor {
        let base: sqlite3_vtab_cursor = unsafe { mem::zeroed() };
        RegexFindAllCursor {
            base,
            requirement: "".into(),
            version_requirement: None,
            current_idx: 0,
        }
    }
}

impl VTabCursor for RegexFindAllCursor {
    fn filter(
        &mut self,
        _idx_num: c_int,
        _idx_str: Option<&str>,
        values: &[*mut sqlite3_value],
    ) -> Result<()> {
        let text = api::value_text(
            values
                .get(0)
                .ok_or_else(|| Error::new_message("expected 1st argument as regex"))?,
        )?;

        self.requirement = text.to_owned();
        self.version_requirement = Some(VersionReq::parse(text).unwrap());
        self.current_idx = 0;
        Ok(())
    }

    fn next(&mut self) -> Result<()> {
        self.current_idx += 1;
        Ok(())
    }

    fn eof(&self) -> bool {
        println!("{}", self.current_idx);
        self.version_requirement
            .as_ref()
            .map_or(true, |req| self.current_idx >= req.comparators.len())
    }

    fn column(&self, context: *mut sqlite3_context, i: c_int) -> Result<()> {
        let m = self
            .version_requirement
            .as_ref()
            .ok_or_else(|| {
                Error::new_message("sqlite-regex internal error: self.match is not defined")
            })?
            .comparators
            .get(self.current_idx)
            .ok_or_else(|| {
                Error::new_message(
                    "sqlite-regex internal error: self.curr greater than matches result",
                )
            })?;

        match column(i) {
            Some(Columns::Op) => {
                let op = match m.op {
                    semver::Op::Exact => "exact",
                    semver::Op::Greater => "greater",
                    semver::Op::GreaterEq => "greater_eq",
                    semver::Op::Less => "less",
                    semver::Op::LessEq => "less_qq",
                    semver::Op::Tilde => "tilde",
                    semver::Op::Caret => "caret",
                    semver::Op::Wildcard => "wildcard",
                    _ => todo!(),
                };
                api::result_text(context, op)?;
            }
            Some(Columns::Major) => {
                match m.major.try_into() {
                    Ok(major) => api::result_int64(context, major),
                    Err(_) => api::result_text(context, m.major.to_string())?,
                };
            }
            Some(Columns::Minor) => match m.minor {
                Some(value) => match value.try_into() {
                    Ok(value) => api::result_int64(context, value),
                    Err(_) => api::result_text(context, value.to_string())?,
                },
                None => api::result_null(context),
            },
            Some(Columns::Patch) => match m.patch {
                Some(value) => match value.try_into() {
                    Ok(value) => api::result_int64(context, value),
                    Err(_) => api::result_text(context, value.to_string())?,
                },
                None => api::result_null(context),
            },
            Some(Columns::Pre) => {
                if m.pre == Prerelease::EMPTY {
                    api::result_null(context)
                } else {
                    api::result_text(context, m.pre.as_str())?
                }
            }
            Some(Columns::Requirement) => api::result_text(context, self.requirement.as_str())?,
            None => (),
        }
        Ok(())
    }

    fn rowid(&self) -> Result<i64> {
        Ok(self.current_idx as i64)
    }
}
