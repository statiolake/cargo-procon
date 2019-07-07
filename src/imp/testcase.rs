use failure_derive::Fail;
use std::io;
use std::path::{Path, PathBuf};

const TEST_FILE: &str = "tests/sample_case.rs";
const TEST_MACRO: &str = "procontest::testcase!(id: $id);";

#[derive(Debug, Fail)]
#[fail(display = "Failed to determine next id.")]
pub struct NextIdError(#[fail(cause)] NextIdErrorKind);

#[derive(Debug, Fail)]
pub enum NextIdErrorKind {
    #[fail(display = "Failed to create `tests` dir.")]
    CreateTestsDirFailed(#[fail(cause)] io::Error),
}

#[derive(Debug, Fail)]
#[fail(display = "Failed to add a new testcase.")]
pub struct AddcaseError(#[fail(cause)] AddcaseErrorKind);

#[derive(Debug, Fail)]
pub enum AddcaseErrorKind {
    #[fail(display = "Failed to remove file `{}`", path_str)]
    RemovingFileFailed {
        #[fail(cause)]
        cause: io::Error,

        path_str: String,
    },

    #[fail(display = "File `{}` already exists.", path_str)]
    FileAlreadyExists { path_str: String },

    #[fail(display = "Writing a testcase failed.")]
    WritingTestcaseFailed(#[fail(cause)] io::Error),

    #[fail(display = "Writing to testfile failed.")]
    WritingTestfileFailed(#[fail(cause)] WriteTestfileErrorKind),
}

#[derive(Debug, Fail)]
#[fail(display = "Failed to delete the testcase.")]
pub struct DelcaseError(#[fail(cause)] DelcaseErrorKind);

#[derive(Debug, Fail)]
pub enum DelcaseErrorKind {
    #[fail(display = "Failed to remove file `{}`.", path_str)]
    RemovingFileFailed {
        #[fail(cause)]
        cause: io::Error,
        path_str: String,
    },

    #[fail(display = "Failed to shift succeeding testcases.")]
    ShiftFailed(#[fail(cause)] io::Error),

    #[fail(display = "Writing to testfile failed.")]
    WritingTestfileFailed(#[fail(cause)] WriteTestfileErrorKind),
}

#[derive(Debug, Fail)]
pub enum WriteTestfileErrorKind {
    #[fail(display = "IO Error for path `{}`", path_str)]
    IOError {
        #[fail(cause)]
        cause: io::Error,
        path_str: String,
    },
}

/*
   For `tests/t**_in.txt`,

   the `index` is `**`,
   the `id` is `t**`,
   the `file_name` is `t**_in.txt`,
   the `file_path` is `tests/t**_in.txt`,
*/
pub fn next_id() -> Result<String, NextIdError> {
    ensure_tests_dir_exists()
        .map_err(NextIdErrorKind::CreateTestsDirFailed)
        .map_err(NextIdError)?;

    let next_id = (1..)
        .map(default_id)
        .find(|cand| !testcase_in_path(cand).exists() && !testcase_out_path(cand).exists())
        .expect("infinite iterator has run out.  This is of course a bug.");

    Ok(next_id)
}

pub fn addcase(id: &str, force: bool, input: &str, output: &str) -> Result<(), AddcaseError> {
    addcase_impl(id, force, input, output).map_err(AddcaseError)
}

fn addcase_impl(id: &str, force: bool, input: &str, output: &str) -> Result<(), AddcaseErrorKind> {
    use std::fs::remove_file;

    let in_path = testcase_in_path(id);
    let out_path = testcase_out_path(id);
    let to_removing_err = |cause: io::Error| AddcaseErrorKind::RemovingFileFailed {
        cause,
        path_str: in_path.display().to_string(),
    };

    if force && in_path.exists() {
        remove_file(&in_path).map_err(to_removing_err)?;
    }

    if force && out_path.exists() {
        remove_file(&out_path).map_err(to_removing_err)?;
    }

    if in_path.exists() {
        return Err(AddcaseErrorKind::FileAlreadyExists {
            path_str: in_path.display().to_string(),
        });
    }

    if out_path.exists() {
        return Err(AddcaseErrorKind::FileAlreadyExists {
            path_str: out_path.display().to_string(),
        });
    }

    write_testcase(&in_path, input).map_err(AddcaseErrorKind::WritingTestcaseFailed)?;
    write_testcase(&out_path, output).map_err(AddcaseErrorKind::WritingTestcaseFailed)?;
    add_to_testfile(id).map_err(AddcaseErrorKind::WritingTestfileFailed)?;

    Ok(())
}

pub fn delcase(id: &str) -> Result<(), DelcaseError> {
    delcase_impl(id).map_err(DelcaseError)
}

fn delcase_impl(id: &str) -> Result<(), DelcaseErrorKind> {
    use std::fs::remove_file;

    // Find the testcase
    let in_path = testcase_in_path(id);
    let out_path = testcase_out_path(id);
    let to_removing_err = |cause: io::Error| DelcaseErrorKind::RemovingFileFailed {
        cause,
        path_str: in_path.display().to_string(),
    };

    // Remove them
    remove_file(&in_path).map_err(to_removing_err)?;
    remove_file(&out_path).map_err(to_removing_err)?;
    remove_from_testfile(id).map_err(DelcaseErrorKind::WritingTestfileFailed)?;

    // if the testcase is a numbered one, shift the succeeding testcases.
    if let Some(idx) = index_of_id(id) {
        shift_testcase_to(idx).map_err(DelcaseErrorKind::ShiftFailed)?;
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

fn testcase_in_path(id: &str) -> PathBuf {
    PathBuf::from(format!("tests/{}_in.txt", id))
}

fn testcase_out_path(id: &str) -> PathBuf {
    PathBuf::from(format!("tests/{}_out.txt", id))
}

fn write_testcase(path: &Path, data: &str) -> io::Result<()> {
    use std::fs::File;
    use std::io::prelude::*;
    use std::io::BufWriter;

    let mut file = BufWriter::new(File::create(path)?);
    file.write_all(data.as_bytes())?;

    Ok(())
}

fn add_to_testfile(id: &str) -> Result<(), WriteTestfileErrorKind> {
    use std::fs::OpenOptions;
    use std::io::prelude::*;

    let content = TEST_MACRO.replace("$id", id);
    let path = Path::new(TEST_FILE);

    let to_err = |cause: io::Error| WriteTestfileErrorKind::IOError {
        cause,
        path_str: path.display().to_string(),
    };

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)
        .map_err(to_err)?;

    file.write_all(content.as_bytes()).map_err(to_err)?;

    Ok(())
}

fn remove_from_testfile(id: &str) -> Result<(), WriteTestfileErrorKind> {
    use std::fs::read_to_string;
    use std::fs::OpenOptions;
    use std::io::prelude::*;

    let content = TEST_MACRO.replace("$id", id);
    let path = Path::new(TEST_FILE);
    let to_err = |cause: io::Error| WriteTestfileErrorKind::IOError {
        cause,
        path_str: path.display().to_string(),
    };

    let entries: Vec<_> = read_to_string(path)
        .map_err(to_err)?
        .lines()
        .filter(|&entry| entry != content)
        .map(|entry| format!("{}\n", entry))
        .collect();

    OpenOptions::new()
        .write(true)
        .open(path)
        .map_err(to_err)?
        .write_all(entries.join("").as_bytes())
        .map_err(to_err)?;

    Ok(())
}

fn shift_testcase_to(idx: usize) -> Result<(), io::Error> {
    use std::fs::{copy, remove_file};
    for i in idx.. {
        let orig_in = testcase_in_path(&default_id(i + 1));
        let orig_out = testcase_out_path(&default_id(i + 1));
        let dest_in = testcase_in_path(&default_id(i));
        let dest_out = testcase_out_path(&default_id(i));

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

fn ensure_tests_dir_exists() -> Result<(), io::Error> {
    use std::fs::create_dir_all;
    create_dir_all("tests")?;
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
