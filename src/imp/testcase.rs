use failure::{ensure, format_err};
use failure::{Error, Fallible};
use failure_derive::Fail;
use std::io;
use std::path::{Path, PathBuf};

const TEST_FILE: &str = "tests/sample_case.rs";
const TEST_MACRO: &str = "procontest::testcase!(id: $id);";

#[derive(Debug, Fail)]
#[fail(display = "Failed to determine next id.")]
pub struct NextIdError(#[fail(cause)] Error);

#[derive(Debug, Fail)]
#[fail(display = "Failed to add a new testcase.")]
pub struct AddcaseError(#[fail(cause)] AddcaseErrorKind);

#[derive(Debug, Fail)]
pub enum AddcaseErrorKind {
    #[fail(display = "File `{}` cannot be removed.", path_str)]
    RemovingFileFailed {
        #[fail(cause)]
        cause: io::Error,

        path_str: String,
    },

    #[fail(display = "File `{}` already exists.", path_str)]
    FileAlreadyExists { path_str: String },
}

#[derive(Debug, Fail)]
#[fail(display = "Failed to delete the testcase.")]
pub struct DelcaseError(#[fail(cause)] DelcaseErrorKind);

#[derive(Debug, Fail)]
pub enum DelcaseErrorKind {}

/*
   For `tests/t**_in.txt`,

   the `index` is `**`,
   the `id` is `t**`,
   the `file_name` is `t**_in.txt`,
   the `file_path` is `tests/t**_in.txt`,
*/
pub fn next_id() -> Result<String, NextIdError> {
    ensure_tests_dir_exists().map_err(NextIdError)?;

    let next_id = (1..)
        .map(default_id)
        .find(|cand| !in_file_path(cand).exists() && !out_file_path(cand).exists())
        .expect("infinite iterator has run out.  This is of course a bug.");

    Ok(next_id)
}

pub fn addcase(id: &str, force: bool, input: &str, output: &str) -> Result<(), AddcaseError> {
    use std::fs::remove_file;

    let in_path = in_file_path(id);
    let out_path = out_file_path(id);

    if force && in_path.exists() {
        remove_file(&in_path).map_err(|cause| {
            AddcaseError(AddcaseErrorKind::RemovingFileFailed {
                cause,
                path_str: in_path.display().to_string(),
            })
        })?;
    }

    if force && out_path.exists() {
        remove_file(&out_path).map_err(|cause| {
            AddcaseError(AddcaseErrorKind::RemovingFileFailed {
                cause,
                path_str: out_path.display().to_string(),
            })
        })?;
    }

    if in_path.exists() {
        return AddcaseError(AddcaseErrorKind::FileAlreadyExists {
            path_str: in_path.display().to_string(),
        });
    }

    ensure!(
        !in_path.exists(),
        "Input file path `{}` already exists.",
        in_path.display()
    );

    ensure!(
        !out_path.exists(),
        "Output file path `{}` already exists.",
        out_path.display()
    );

    write_testcase(&in_path, input)?;
    write_testcase(&out_path, output)?;
    add_testcase_to_file(id)?;

    Ok(())
}

pub fn delcase(id: &str) -> Fallible<()> {
    delcase_impl(id).map_err(|e| DelcaseError(e).into())
}

fn delcase_impl(id: &str) -> Fallible<()> {
    use std::fs::remove_file;

    // Find the testcase
    let in_path = in_file_path(id);
    let out_path = out_file_path(id);

    // Remove them
    remove_file(&in_path)
        .map_err(|e| format_err!("failed to remove `{}`: {}", in_path.display(), e))?;
    remove_file(&out_path)
        .map_err(|e| format_err!("failed to remove `{}`: {}", out_path.display(), e))?;
    remove_testcase_from_file(id)?;

    // if the testcase is a numbered one, shift the following testcases.
    if let Some(idx) = index_of_id(id) {
        shift_testcase_to(idx)?;
    }

    Ok(())
}

fn default_id(index: usize) -> String {
    format!("t{}", index)
}

fn index_of_id(id: &str) -> Option<usize> {
    if !id.starts_with('t') {
        return None;
    }

    id[1..].parse().ok()
}

fn in_file_path(id: &str) -> PathBuf {
    PathBuf::from(format!("tests/{}_in.txt", id))
}

fn out_file_path(id: &str) -> PathBuf {
    PathBuf::from(format!("tests/{}_out.txt", id))
}

fn write_testcase(path: &Path, data: &str) -> Fallible<()> {
    use std::fs::File;
    use std::io::prelude::*;
    use std::io::BufWriter;

    let mut file = BufWriter::new(File::create(path)?);
    file.write_all(data.as_bytes())?;

    Ok(())
}

fn add_testcase_to_file(id: &str) -> Fallible<()> {
    use std::fs::OpenOptions;
    use std::io::prelude::*;

    let content = TEST_MACRO.replace("$id", id);
    let path = Path::new(TEST_FILE);

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)
        .map_err(|e| format_err!("Cannot open the test file `{}`: {}.", path.display(), e))?;

    file.write_all(content.as_bytes())
        .map_err(|e| format_err!("Cannot write to the test file `{}`: {}", path.display(), e))?;

    Ok(())
}

fn remove_testcase_from_file(id: &str) -> Fallible<()> {
    use std::fs::read_to_string;
    use std::fs::OpenOptions;
    use std::io::prelude::*;

    let content = TEST_MACRO.replace("$id", id);
    let path = Path::new(TEST_FILE);
    let disp = path.display();

    let entries: Vec<_> = read_to_string(path)
        .map_err(|e| format_err!("Cannot open the test file `{}`: `{}`", disp, e))?
        .lines()
        .filter(|&entry| entry != content)
        .map(|entry| format!("{}\n", entry))
        .collect();

    OpenOptions::new()
        .write(true)
        .open(path)
        .map_err(|e| format_err!("Cannot open the test file `{}`: `{}`", disp, e))?
        .write_all(entries.join("").as_bytes())
        .map_err(|e| format_err!("Failed to write the test file `{}`: `{}`", disp, e))?;

    Ok(())
}

fn shift_testcase_to(idx: usize) -> Fallible<()> {
    use std::fs::{copy, remove_file};
    for i in idx.. {
        let orig_in = in_file_path(&default_id(i + 1));
        let orig_out = out_file_path(&default_id(i + 1));
        let dest_in = in_file_path(&default_id(i));
        let dest_out = out_file_path(&default_id(i));

        assert!(
            !dest_in.exists() && !dest_out.exists(),
            "The destination file must not exist.  This is a bug."
        );

        // if the next test case doesn't exist, the operation was finished.
        if !orig_in.exists() && !orig_out.exists() {
            break;
        }

        // move the input file
        if orig_in.exists() {
            copy(&orig_in, &dest_in)?;
            remove_file(&orig_in)?;
        }

        // move the output file
        if orig_out.exists() {
            copy(&orig_out, &dest_out)?;
            remove_file(&orig_out)?;
        }
    }

    Ok(())
}

fn ensure_tests_dir_exists() -> Fallible<()> {
    use std::fs::create_dir_all;
    create_dir_all("tests").map_err(|e| format_err!("Cannot create `tests` directory: {}", e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inter_transform() {
        for i in 0..100 {
            assert_eq!(index_of_id(&default_id(i)), Some(i));
        }
    }
}
