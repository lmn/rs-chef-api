use api_client::ApiClient;
use utils::decode_list;
use errors::*;

// Client Structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Client {
    #[serde(default)]
    pub name: String,
    #[serde(default)] clientname: String,
    #[serde(default)] validator: bool,
    #[serde(default)] orgname: String,
    #[serde(default)] json_class: String,
    #[serde(default)] chef_type: String,
}

impl Client {
    pub fn show(client: &ApiClient, name: String) -> Result<Client> {
        let org = &client.config.organization_path();
        let path = format!("{}/clients/{}", org, name);
        client.get::<Client>(path.as_ref())
    }

    pub fn delete(client: &ApiClient, name: String) -> Result<Client> {
        let org = &client.config.organization_path();
        let path = format!("{}/clients/{}", org, name);
        client.delete::<Client>(path.as_ref())
    }
}


pub fn delete_client(client: &ApiClient, name: &str) -> Result<Client> {
    let org = &client.config.organization_path();
    let path = format!("{}/clients/{}", org, name);
    client.delete::<Client>(path.as_ref())
}

// Clients Structure
#[derive(Debug)]
pub struct Clients {
    count: usize,
    clients: Vec<String>,
    client: ApiClient,
}

impl Clients {
    pub fn list(client: &ApiClient) -> Clients {
        let org = &client.config.organization_path();
        let path = format!("{}/clients", org);
        client
            .get(path.as_ref())
            .and_then(decode_list)
            .and_then(|list| {
                Ok(Clients {
                    count: 0,
                    clients: list,
                    client: client.clone(),
                })
            })
            .unwrap()
    }
    pub fn show(client: &ApiClient, name: String) -> Result<Client> {
        let org = &client.config.organization_path();
        let path = format!("{}/clients/{}", org, name);
        client.get(path.as_ref())
    }
    pub fn delete(client: &ApiClient, name: String) -> Result<Client> {
        let org = &client.config.organization_path();
        let path = format!("{}/clients/{}", org, name);
        client.delete(path.as_ref())
    }
}

// Itenarator for Clients
impl Iterator for Clients {
    type Item = Result<Client>;

    fn count(self) -> usize {
        self.clients.len()
    }

    fn next(&mut self) -> Option<Self::Item> {
        if self.clients.len() >= 1 {
            Some(Client::show(&self.client, self.clients.remove(0)))
        } else {
            None
        }
    }
}
