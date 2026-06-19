use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

const APP_DIRECTORY: &str = "proteus-player";
const RECENT_FILES_NAME: &str = "recent-files.json";

pub(crate) fn load() -> Result<Vec<PathBuf>, String> {
    let path = storage_path()?;

    match fs::read(&path) {
        Ok(contents) => serde_json::from_slice(&contents)
            .map_err(|error| format!("could not parse {}: {error}", path.display())),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(Vec::new()),
        Err(error) => Err(format!("could not read {}: {error}", path.display())),
    }
}

pub(crate) fn save(files: &[PathBuf]) -> Result<(), String> {
    let path = storage_path()?;
    let directory = path
        .parent()
        .expect("the recent-files storage path always has a parent directory");

    fs::create_dir_all(directory)
        .map_err(|error| format!("could not create {}: {error}", directory.display()))?;

    let contents = serde_json::to_vec(files)
        .map_err(|error| format!("could not serialize recent files: {error}"))?;
    fs::write(&path, contents)
        .map_err(|error| format!("could not write {}: {error}", path.display()))
}

fn storage_path() -> Result<PathBuf, String> {
    dirs::data_local_dir()
        .map(|directory| directory.join(APP_DIRECTORY).join(RECENT_FILES_NAME))
        .ok_or_else(|| "could not determine the app data directory".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn recent_files_are_stored_as_json_paths() {
        let files = vec![
            PathBuf::from("/music/first.prot"),
            PathBuf::from("/music/second.mp3"),
        ];
        let serialized = serde_json::to_vec(&files).expect("paths should serialize");

        let restored: Vec<PathBuf> =
            serde_json::from_slice(&serialized).expect("paths should deserialize");

        assert_eq!(restored, files);
    }

    #[test]
    fn storage_path_is_under_the_app_data_directory() {
        let path = storage_path().expect("a desktop platform should have an app data directory");

        assert_eq!(path.file_name(), Some(RECENT_FILES_NAME.as_ref()));
        assert_eq!(
            path.parent().and_then(Path::file_name),
            Some(APP_DIRECTORY.as_ref())
        );
    }
}
