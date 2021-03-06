use api_client::ApiClient;
use serde_json;
use serde_json::Value;
use std::collections::HashMap;
use std::io;
use std::io::{Cursor, Read};
use std::io::ErrorKind as IoErrorKind;
use utils::decode_list;
use errors::*;

chef_json_type!(DataBagItemJsonClass, "Chef::DataBagItem");
chef_json_type!(DataBagItemChefType, "data_bag_item");

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DataBagItem {
    id: Option<String>,
    #[serde(default)] data_bag: Option<String>,
    #[serde(default)] chef_type: DataBagItemChefType,
    #[serde(default)] json_class: DataBagItemJsonClass,
    #[serde(default)] pub raw_data: HashMap<String, Value>,
}

impl Read for DataBagItem {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if let Ok(data_bag_item) = serde_json::to_vec(self) {
            let mut data_bag_item = Cursor::new(data_bag_item.as_ref() as &[u8]);
            Read::read(&mut data_bag_item, buf)
        } else {
            Err(io::Error::new(
                IoErrorKind::InvalidData,
                "Failed to convert data bag item to JSON",
            ))
        }
    }
}

impl DataBagItem {
    pub fn new<N, D>(id: N, data_bag: D) -> Self
    where
        N: Into<String>,
        D: Into<String>,
    {
        DataBagItem {
            id: Some(id.into()),
            data_bag: Some(data_bag.into()),
            ..Default::default()
        }
    }

    pub fn fetch<N, D>(client: &ApiClient, data_bag: D, id: N) -> Result<DataBagItem>
    where
        N: Into<String>,
        D: Into<String>,
    {
        let org = &client.config.organization_path();
        let path = format!("{}/data/{}/{}", org, data_bag.into(), id.into());
        client.get::<DataBagItem>(path.as_ref())
    }

    pub fn id(&self) -> Result<String> {
        if let Some(id) = self.raw_data.get("id") {
            serde_json::from_value(id.clone()).chain_err(|| "Failed to fetch field")
        } else {
            Err(ErrorKind::KeyMissingError("id".to_owned()).into())
        }
    }

    pub fn save(&self, client: &ApiClient) -> Result<DataBagItem> {
        let id = try!(self.id());
        let data_bag = &self.data_bag.clone().unwrap();
        let org = &client.config.organization_path();
        let path = format!("{}/data/{}/{}", org, data_bag, id);
        client.put::<&DataBagItem, DataBagItem>(path.as_ref(), &self)
    }

    pub fn delete(&self, client: &ApiClient) -> Result<DataBagItem> {
        let id = try!(self.id());
        let data_bag = &self.data_bag.clone().unwrap();
        let org = &client.config.organization_path();
        let path = format!("{}/data/{}/{}", org, data_bag, id);
        client.delete::<DataBagItem>(path.as_ref())
    }

    pub fn from_json<R>(r: R) -> Result<DataBagItem>
    where
        R: Read,
    {
        Ok(try!(serde_json::from_reader::<R, DataBagItem>(r)))
    }
}

pub fn delete_data_bag_item<D, N>(client: &ApiClient, data_bag: D, id: N) -> Result<DataBagItem>
where
    D: Into<String>,
    N: Into<String>,
{
    let org = &client.config.organization_path();
    let path = format!("{}/data/{}/{}", org, data_bag.into(), id.into());
    client.delete::<DataBagItem>(path.as_ref())
}

#[derive(Debug)]
pub struct DataBagItemList {
    count: usize,
    data_bag: String,
    data_bag_items: Vec<String>,
    client: ApiClient,
}

impl DataBagItemList {
    pub fn new<D: Into<String>>(client: &ApiClient, data_bag: D) -> Self {
        let org = &client.config.organization_path();
        let db = data_bag.into();
        let path = format!("{}/data/{}", org, &db);
        client
            .get(path.as_ref())
            .and_then(decode_list)
            .and_then(|list| {
                Ok(DataBagItemList {
                    data_bag: db,
                    data_bag_items: list,
                    count: 0,
                    client: client.clone(),
                })
            })
            .unwrap()
    }
}

impl Iterator for DataBagItemList {
    type Item = Result<DataBagItem>;

    fn count(self) -> usize {
        self.data_bag_items.len()
    }

    fn next(&mut self) -> Option<Self::Item> {
        if self.data_bag_items.len() >= 1 {
            Some(DataBagItem::fetch(
                &self.client,
                self.data_bag.clone(),
                self.data_bag_items.remove(0),
            ))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    // use super::DataBagItem;
    // use std::fs::File;

    //     #[test]
    //     fn test_data_bag_item_from_file() {
    //         let fh = File::open("fixtures/data_bag_item.json").unwrap();
    //         let data_bag_item = DataBagItem::from_json(fh).unwrap();
    //         assert_eq!(data_bag_item.id(), "test")
    //     }
}
