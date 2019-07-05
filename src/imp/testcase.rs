use failure::Fallible;
use failure::{ensure, format_err};
use std::path::{Path, PathBuf};

const TEST_FILE: &str = "tests/sample_case.rs";
const TEST_MACRO: &str = "procontest::testcase!(id: $id);";

/*
   For `tests/t**_in.txt`,

   the `index` is `**`,
   the `id` is `t**`,
   the `file_name` is `t**_in.txt`,
   the `file_path` is `tests/t**_in.txt`,
*/
pub fn next_id() -> Fallible<String> {
    ensure_tests_dir_exists()?;

    let next_id = (1..)
        .map(default_id)
        .find(|cand| !in_file_path(cand).exists() && !out_file_path(cand).exists())
        .expect("infinite iterator has run out.  This is of course a bug.");

    Ok(next_id)
}

pub fn addcase(id: &str, force: bool, input: &str, output: &str) -> Fallible<()> {
    use std::fs::remove_file;

    let in_path = in_file_path(id);
    let out_path = out_file_path(id);

    if force && in_path.exists() {
        remove_file(&in_path)?;
    }

    if force && out_path.exists() {
        remove_file(&out_path)?;
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

fn default_id(index: usize) -> String {
    format!("t{}", index)
}

#[cfg(test)]
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
