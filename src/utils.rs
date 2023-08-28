use semver::Version;
use sqlite_loadable::prelude::*;
use sqlite_loadable::{
    api,
    errors::{Error, Result},
};
use std::os::raw::c_void;

const SEMVER_VERSION_POINTER_NAME: &[u8] = b"semver_version0\0";

pub fn value_semver_version(value: &*mut sqlite3_value) -> Result<*mut Version> {
    unsafe {
        if let Some(version) = api::value_pointer(value, SEMVER_VERSION_POINTER_NAME) {
            return Ok(version);
        }
    }
    let text = api::value_text(value)?;
    let boxed = Box::new(Version::parse(text).map_err(|err| {
        Error::new_message(format!("Error parsing semver Version: {}", err).as_str())
    })?);
    Ok(Box::into_raw(boxed))
}

pub enum SemverVersionInputType {
    Pointer,
    TextInitial(usize),
    GetAuxdata,
}
pub fn semver_version_from_value_or_cache(
    context: *mut sqlite3_context,
    values: &[*mut sqlite3_value],
    at: usize,
) -> Result<(*mut Version, SemverVersionInputType)> {
    let value = values
        .get(at)
        .ok_or_else(|| Error::new_message("expected 1st argument as pattern"))?;

    // Step 1: If the value is a pointer result of semver_version(),
    // just use that.
    unsafe {
        if let Some(regex) = api::value_pointer(value, SEMVER_VERSION_POINTER_NAME) {
            return Ok((regex, SemverVersionInputType::Pointer));
        }
    }

    // Step 2: If sqlite3_get_auxdata returns a pointer,
    // then use that.

    let auxdata = api::auxdata_get(context, at as i32);
    if !auxdata.is_null() {
        Ok((
            auxdata.cast::<Version>(),
            SemverVersionInputType::GetAuxdata,
        ))
    } else {
        // Step 3: if a string is passed in, then try to make
        // a Version from that, and return a flag to call sqlite3_set_auxdata

        let text = api::value_text(value)?;
        let boxed = Box::new(Version::parse(text).unwrap());
        Ok((
            Box::into_raw(boxed),
            SemverVersionInputType::TextInitial(at),
        ))
    }
}

pub fn cleanup_semver_version_value_cached(
    context: *mut sqlite3_context,
    version: *mut Version,
    input_type: SemverVersionInputType,
) {
    match input_type {
        SemverVersionInputType::Pointer => (),
        SemverVersionInputType::GetAuxdata => {}
        SemverVersionInputType::TextInitial(at) => api::auxdata_set(
            context,
            at as i32,
            version.cast::<c_void>(),
            Some(cleanup_version),
        ),
    }
}
unsafe extern "C" fn cleanup_version(arg: *mut c_void) {
    drop(Box::from_raw(arg.cast::<*mut Version>()))
}

pub fn result_semver_version(context: *mut sqlite3_context, version: Version) {
    api::result_pointer(context, SEMVER_VERSION_POINTER_NAME, version)
}
