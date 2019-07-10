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
    UpdatingTestfileFailed(#[fail(cause)] UpdatingTestfileErrorKind),
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

    #[fail(display = "Updating the testfile failed.")]
    UpdatingTestfileFailed(#[fail(cause)] UpdatingTestfileErrorKind),
}

#[derive(Debug, Fail)]
pub enum UpdatingTestfileErrorKind {
    #[fail(display = "Failed to read directory `{}`", path_str)]
    ReadDirError {
        #[fail(cause)]
        cause: io::Error,
        path_str: String,
    },
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
    update_testfile().map_err(AddcaseErrorKind::UpdatingTestfileFailed)?;

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

    // if the testcase is a numbered one, shift the succeeding testcases.
    if let Some(idx) = index_of_id(id) {
        shift_testcase_to(idx).map_err(DelcaseErrorKind::ShiftFailed)?;
    }

    update_testfile().map_err(DelcaseErrorKind::UpdatingTestfileFailed)?;

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

fn update_testfile() -> Result<(), UpdatingTestfileErrorKind> {
    use std::fs;
    let file_path = Path::new(TEST_FILE);
    let dir_path = file_path
        .parent()
        .expect("Failed to get the test directory path.  This is a bug.");

    let testcases = fs::read_dir(dir_path)
        .map_err(|cause| UpdatingTestfileErrorKind::ReadDirError {
            cause,
            path_str: dir_path.display().to_string(),
        })?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let file_name = entry.file_name().into_string().ok()?;
            if file_name.ends_with("_in.txt") {
                Some(file_name[0..file_name.len() - 7].to_string())
            } else {
                None
            }
        });

    let testcases: Vec<_> = testcases
        .map(|entry| TEST_MACRO.replace("$id", &entry))
        .map(|entry| format!("{}\n", entry))
        .collect();
    let testcases = testcases.join("");

    fs::write(file_path, testcases).map_err(|cause| UpdatingTestfileErrorKind::IOError {
        cause,
        path_str: file_path.display().to_string(),
    })?;

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
