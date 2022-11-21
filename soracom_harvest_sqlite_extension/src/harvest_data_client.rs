//! Represents Soracom Harvest Client and its associated data.

use soracom_harvest_api_client::{
    client::{Data, SoracomHarvestClient},
    error::SoracomHarvestClientError,
};
use typed_builder::TypedBuilder;

/// Harvest Data
#[derive(TypedBuilder)]
pub struct HarvestDataClient {
    #[builder]
    client: SoracomHarvestClient,
    #[builder(default)]
    data: Vec<Data>,
    #[builder(default)]
    imsi: String,
    #[builder(default)]
    from: i64,
    #[builder(default)]
    to: i64,
    #[builder(default)]
    limit: u32,
}

impl HarvestDataClient {
    /// Authenticate with provided credential and get data.
    pub fn open(&mut self) -> Result<(), SoracomHarvestClientError> {
        let client = self.client.auth()?;

        self.data = client.get_data_entries(
            &self.imsi,
            Some(self.from),
            Some(self.to),
            Some(self.limit),
        )?;

        Ok(())
    }

    /// Get reader for the data.
    pub fn get_reader(&mut self) -> HarvestDataReader {
        HarvestDataReader::new(self.data.clone()) // it should not be cloned, but for simplicity.
    }
}

/// Reader for given data.
pub struct HarvestDataReader {
    data: Vec<Data>,
    current_index: usize,
}

impl HarvestDataReader {
    /// Returns a new reader for given data.
    pub fn new(data: Vec<Data>) -> Self {
        HarvestDataReader {
            data,
            current_index: 0,
        }
    }

    /// Get current index.
    pub fn get_index(&self) -> u32 {
        self.current_index as u32
    }

    /// Increment index.
    pub fn move_next(&mut self) {
        self.current_index += 1;
    }

    /// Returns if the current index has a data.
    pub fn has_value(&self) -> bool {
        self.data.get(self.current_index).is_some()
    }

    /// Get value of the current index.
    pub fn get_value(&self, i: usize) -> String {
        match self.data.get(self.current_index) {
            None => "".to_string(),
            Some(d) => match i {
                0 => d.time.to_string(),
                1 => d.content_type.clone(),
                _ => d.content.clone(),
            },
        }
    }
}
