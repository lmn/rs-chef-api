use api_client::ApiClient;
use serde_json;
use std::io;
use std::io::{Cursor, Read};
use std::io::ErrorKind as IoErrorKind;
use errors::*;

chef_json_type!(DataBagJsonClass, "Chef::DataBag");
chef_json_type!(DataBagChefType, "data_bag");

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DataBag {
    #[serde(default)] pub name: Option<String>,
    #[serde(default)] chef_type: DataBagChefType,
    #[serde(default)] json_class: DataBagJsonClass,
    #[serde(default)] id: Option<usize>,
}

impl Read for DataBag {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if let Ok(data_bag) = serde_json::to_vec(self) {
            let mut data_bag = Cursor::new(data_bag.as_ref() as &[u8]);
            Read::read(&mut data_bag, buf)
        } else {
            Err(io::Error::new(
                IoErrorKind::InvalidData,
                "Failed to convert data bag to JSON",
            ))
        }
    }
}

impl DataBag {
    pub fn new<S>(name: S) -> DataBag
    where
        S: Into<String>,
    {
        DataBag {
            name: Some(name.into()),
            ..Default::default()
        }
    }

    pub fn fetch<S: Into<String>>(client: &ApiClient, name: S) -> Result<DataBag> {
        let org = &client.config.organization_path();
        let path = format!("{}/data/{}", org, name.into());
        client.get::<DataBag>(path.as_ref())
    }

    pub fn save(&self, client: &ApiClient) -> Result<DataBag> {
        let org = &client.config.organization_path();
        let path = format!("{}/data", org);
        client.post::<&DataBag, DataBag>(path.as_ref(), &self)
    }

    pub fn from_json<R>(r: R) -> Result<DataBag>
    where
        R: Read,
    {
        Ok(try!(serde_json::from_reader::<R, DataBag>(r)))
    }
}
