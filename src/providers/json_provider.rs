use std::{
    fs::{File, OpenOptions},
    path::PathBuf,
};

use time::{format_description, OffsetDateTime};

use crate::models::Entries;

use super::{Provider, ProviderError};

pub struct JsonProvider {
    file: PathBuf,
}

impl JsonProvider {
    pub fn new(mut file: PathBuf) -> Self {
        if !file.ends_with(".json") {
            file.set_extension("json");
        }
        return JsonProvider { file };
    }

    pub fn new_today() -> Result<Self, ProviderError> {
        let format = format_description::parse("[year]-[month]-[day]")?;
        let date_str = OffsetDateTime::now_utc().date().format(&format)?;
        let file = PathBuf::from(date_str);
        Ok(JsonProvider::new(file))
    }
}

impl Provider for JsonProvider {
    fn read(&self) -> Result<Entries, ProviderError> {
        log::debug!("read using json provider");
        let mut entries = Entries::new();

        if self.file.exists() {
            log::debug!("file found appending data: {:?}", &self.file);
            let f = File::open(&self.file)?;
            entries = serde_json::from_reader(f)?;
            entries.sort_by();
        } else {
            log::info!("file not yet created: {:?}", &self.file);
            // TODO: invoke takeover- here? or use some prepending logic?
        }
        Ok(entries)
    }

    fn write(&self, entries: &Entries) -> Result<(), ProviderError> {
        log::debug!("write using json provider");
        let w = OpenOptions::new()
            .create(true)
            .write(true)
            .append(false)
            .truncate(false)
            .open(&self.file)?;
        log::debug!("write to file: {:?}", &self.file);
        Ok(serde_json::to_writer(w, &entries)?)
    }
}

#[cfg(test)]
mod tests {
    use super::JsonProvider;
    use std::path::PathBuf;

    #[test]
    fn should_generate_file_name_for_today(){
        let provider = JsonProvider::new_today().unwrap();
        assert!(provider.file.as_os_str().to_str().unwrap().ends_with("json"))
    }

    #[test]
    fn should_append_file_extension(){
        let path = PathBuf::from("sample");
        let provider = JsonProvider::new(path);
        assert_eq!("sample.json", provider.file.as_os_str())
    }

    #[test]
    fn should_not_append_file_extension(){
        let path = PathBuf::from("sample2.json");
        let provider = JsonProvider::new(path);
        assert_eq!("sample2.json", provider.file.as_os_str())
    }

    // TODO: add integration tests for file write and read with test context and tempdir
}
