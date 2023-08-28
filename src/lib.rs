mod requirements;
mod utils;

use crate::{requirements::SemverRequrementsTable, utils::*};
use semver::{BuildMetadata, Prerelease, Version, VersionReq};
use sqlite_loadable::{
    api, define_scalar_function, define_table_function,
    errors::{Error, Result},
    FunctionFlags,
};
use sqlite_loadable::{define_collation, prelude::*};
use std::cmp::Ordering;

pub fn semver_version(context: *mut sqlite3_context, _values: &[*mut sqlite3_value]) -> Result<()> {
    api::result_text(context, format!("v{}", env!("CARGO_PKG_VERSION")))?;
    Ok(())
}
pub fn semver_version_text(
    context: *mut sqlite3_context,
    values: &[*mut sqlite3_value],
) -> Result<()> {
    let version = semver_version_constructor(context, values)?;
    api::result_text(context, version.to_string())?;
    Ok(())
}
pub fn semver_version_pointer(
    context: *mut sqlite3_context,
    values: &[*mut sqlite3_value],
) -> Result<()> {
    let version = semver_version_constructor(context, values)?;
    result_semver_version(context, version);
    Ok(())
}

fn semver_version_constructor(
    context: *mut sqlite3_context,
    values: &[*mut sqlite3_value],
) -> Result<Version> {
    let major: u64 = api::value_int64(values.get(0).ok_or_else(|| Error::new_message("asdf"))?)
        .try_into()
        .map_err(|err| Error::new_message(format!("major cannot be {}", err)))?;
    let minor = api::value_int64(values.get(1).ok_or_else(|| Error::new_message("asdf"))?);
    let patch = api::value_int64(values.get(2).ok_or_else(|| Error::new_message("asdf"))?);
    let pre = match values.get(3) {
        Some(v) => Prerelease::new(api::value_text(v)?).unwrap(),
        None => Prerelease::EMPTY,
    };
    let build = match values.get(4) {
        Some(v) => BuildMetadata::new(api::value_text(v)?).unwrap(),
        None => BuildMetadata::EMPTY,
    };
    let version = Version {
        major,
        minor: minor as u64,
        patch: patch as u64,
        pre,
        build,
    };
    Ok(version)
}

pub fn semver_debug(context: *mut sqlite3_context, _values: &[*mut sqlite3_value]) -> Result<()> {
    api::result_text(
        context,
        format!(
            "Version: v{}
      Source: {}
      ",
            env!("CARGO_PKG_VERSION"),
            env!("GIT_HASH")
        ),
    )?;
    Ok(())
}

pub fn semver_matches(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    let (version, input_type) = semver_version_from_value_or_cache(context, values, 0)?;
    let req = VersionReq::parse(api::value_text(values.get(1).unwrap())?).unwrap();
    api::result_bool(context, req.matches(unsafe { &*version }));
    cleanup_semver_version_value_cached(context, version, input_type);
    Ok(())
}
pub fn semver_gt(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    let (version_a, input_a) = semver_version_from_value_or_cache(context, values, 0)?;
    let (version_b, input_b) = semver_version_from_value_or_cache(context, values, 1)?;
    api::result_bool(context, version_a > version_b);
    cleanup_semver_version_value_cached(context, version_a, input_a);
    cleanup_semver_version_value_cached(context, version_b, input_b);
    Ok(())
}

fn compare(a: &[u8], b: &[u8]) -> i32 {
    let a = std::str::from_utf8(a);
    let b = std::str::from_utf8(b);
    let (a, b) = match (a, b) {
        (Ok(a), Ok(b)) => (a, b),
        _ => return -1,
    };

    let a = Version::parse(a.strip_prefix('v').unwrap_or(a));
    let b = Version::parse(b.strip_prefix('v').unwrap_or(b));
    match (a, b) {
        (Ok(a), Ok(b)) => match a.cmp(&b) {
            Ordering::Equal => 0,
            Ordering::Greater => 1,
            Ordering::Less => -1,
        },
        _ => -1,
    }
}
#[sqlite_entrypoint]
pub fn sqlite3_semver_init(db: *mut sqlite3) -> Result<()> {
    let flags = FunctionFlags::UTF8 | FunctionFlags::DETERMINISTIC;

    define_collation(db, "semver", compare)?;

    define_scalar_function(db, "semver_version", 0, semver_version, flags)?;
    define_scalar_function(db, "semver_debug", 0, semver_debug, flags)?;
    define_scalar_function(db, "semver_version", 3, semver_version_text, flags)?;
    define_scalar_function(db, "semver_version", 4, semver_version_text, flags)?;
    define_scalar_function(db, "semver_version", 5, semver_version_text, flags)?;
    define_scalar_function(
        db,
        "semver_version_pointer",
        3,
        semver_version_pointer,
        flags,
    )?;
    define_scalar_function(
        db,
        "semver_version_pointer",
        4,
        semver_version_pointer,
        flags,
    )?;
    define_scalar_function(
        db,
        "semver_version_pointer",
        5,
        semver_version_pointer,
        flags,
    )?;
    define_scalar_function(db, "semver_matches", 2, semver_matches, flags)?;
    define_scalar_function(db, "semver_gt", 2, semver_gt, flags)?;
    define_table_function::<SemverRequrementsTable>(db, "semver_requirements", None)?;
    Ok(())
}
