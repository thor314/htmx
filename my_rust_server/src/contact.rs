#![allow(dead_code)]
use std::{collections::HashMap, sync::Arc};

use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

#[derive(Debug, Default, Serialize)]
pub struct Contacts(pub HashMap<usize, Contact>);

impl Contacts {
  // validate the new contact has a new unique id and email
  pub fn validate(&self, contact: &Contact) -> bool {
    for c in self.0.values() {
      if c.email == contact.email || c.id == contact.id {
        return false;
      }
    }
    true
  }

  // create a new contact
  pub fn insert(&mut self, contact: Contact) -> bool {
    if self.validate(&contact) {
      self.0.insert(contact.id.unwrap(), contact);
      true
    } else {
      false
    }
  }

  pub fn delete(&mut self, id: usize) -> bool { self.0.remove(&id).is_some() }

  pub fn len(&self) -> usize { self.0.len() }

  pub fn all(&self) -> Vec<Contact> { self.0.values().cloned().collect() }

  pub fn search(&self, search: &str) -> Vec<Contact> {
    self
      .0
      .values()
      .filter(|c| {
        c.first.contains(search)
          || c.last.contains(search)
          || c.email.contains(search)
          || c.phone.contains(search)
      })
      .cloned()
      .collect()
  }

  pub fn get(&self, id: usize) -> Option<Contact> { self.0.get(&id).cloned() }

  // open 'contacts.json' and load the contacts, creating a Contacts list
  pub fn load_db() -> Self {
    let mut contacts = Contacts::default();
    let file = std::fs::File::open("contacts.json").unwrap();
    let reader = std::io::BufReader::new(file);
    let contacts_json: Vec<Contact> = serde_json::from_reader(reader).unwrap();
    for contact in contacts_json {
      contacts.insert(contact);
    }
    contacts
  }

  pub fn save_db(&self) {
    let file = std::fs::File::create("contacts.json").unwrap();
    let writer = std::io::BufWriter::new(file);
    serde_json::to_writer(writer, &self.all()).unwrap();
  }
}

pub struct DatabaseConnection(pub Arc<Mutex<Contacts>>);

#[async_trait]
impl<S> FromRequestParts<S> for DatabaseConnection
where S: Send + Sync
{
  type Rejection = ();

  // (StatusCode, String);

  async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
    let pool = Contacts::load_db();
    // let conn = pool.acquire().await.map_err(internal_error)?;

    Ok(DatabaseConnection(Arc::new(Mutex::new(pool))))
  }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Contact {
  id:    Option<usize>,
  first: String,
  last:  String,
  email: String,
  phone: String,
}

impl Contact {
  pub fn new(id: Option<usize>, first: &str, last: &str, email: &str, phone: &str) -> Self {
    {
      Self {
        id,
        first: first.to_string(),
        last: last.to_string(),
        email: email.to_string(),
        phone: phone.to_string(),
      }
    }
  }

  pub fn update(&mut self, first: &str, last: &str, email: &str, phone: &str) {
    self.first = first.to_string();
    self.last = last.to_string();
    self.email = email.to_string();
    self.phone = phone.to_string();
  }
}
