use shared::{Contact, ContactStore, NewContact, StoreError, validate_contact};

const CREATE_CONTACTS_TABLE: &str = "
    CREATE TABLE IF NOT EXISTS contacts (
        id          INTEGER PRIMARY KEY,
        first_name  TEXT NOT NULL,
        last_name   TEXT NOT NULL,
        company     TEXT NOT NULL,
        email       TEXT NOT NULL,
        active      INTEGER NOT NULL
    )";

const INSERT_CONTACT: &str = "
    INSERT INTO contacts (first_name, last_name, company, email, active)
    VALUES (?1, ?2, ?3, ?4, ?5)";

const DELETE_CONTACT: &str = "
    DELETE FROM contacts WHERE id = ?1";

const UPDATE_CONTACT: &str = "
    UPDATE contacts SET first_name = ?1, last_name = ?2, company = ?3, email = ?4 WHERE id = ?5";

const SELECT_CONTACTS: &str = "
    SELECT id, first_name, last_name, company, email, active FROM contacts";

#[derive(Debug)]
pub struct SqliteStore {
    connection: rusqlite::Connection,
    contacts: Vec<Contact>,
}

impl SqliteStore {
    pub fn open(file_path: &str) -> Result<Self, StoreError> {
        let connection = rusqlite::Connection::open(file_path)
            .map_err(|e| StoreError::Database(e.to_string()))?;

        connection
            .execute(CREATE_CONTACTS_TABLE, [])
            .map_err(|e| StoreError::Database(e.to_string()))?;

        let contacts = {
            let mut stmt = connection
                .prepare(SELECT_CONTACTS)
                .map_err(|e| StoreError::Database(e.to_string()))?;

            let rows = stmt
                .query_map([], |row| {
                    Ok(Contact {
                        id: row.get(0)?,
                        first_name: row.get(1)?,
                        last_name: row.get(2)?,
                        company: row.get(3)?,
                        email: row.get(4)?,
                        active: row.get(5)?,
                    })
                })
                .map_err(|e| StoreError::Database(e.to_string()))?;

            rows.collect::<Result<Vec<Contact>, _>>()
                .map_err(|e| StoreError::Database(e.to_string()))?
        };

        Ok(Self { connection, contacts })
    }
}

impl ContactStore for SqliteStore {
    fn list(&self) -> &[Contact] {
        &self.contacts
    }

    fn create(&mut self, input: NewContact) -> Result<Contact, StoreError> {
        validate_contact(&input)?;

        let first_name = input.first_name.trim().to_string();
        let last_name = input.last_name.trim().to_string();

        self.connection
            .execute(
                INSERT_CONTACT,
                rusqlite::params![first_name, last_name, input.company, input.email, true],
            )
            .map_err(|e| StoreError::Database(e.to_string()))?;

        let id = self.connection.last_insert_rowid() as u32;

        let new = Contact {
            id,
            first_name,
            last_name,
            company: input.company,
            email: input.email,
            active: true,
        };

        self.contacts.push(new.clone());
        Ok(new)
    }

    fn delete(&mut self, id: u32) -> Result<(), StoreError> {
        let rows = self
            .connection
            .execute(DELETE_CONTACT, rusqlite::params![id])
            .map_err(|e| StoreError::Database(e.to_string()))?;

        if rows == 0 {
            return Err(StoreError::NotFound(id));
        }
        self.contacts.retain(|c| c.id != id);
        Ok(())
    }

    fn update(&mut self, id: u32, input: &NewContact) -> Result<Contact, StoreError> {
        validate_contact(input)?;

        let first_name = input.first_name.trim().to_string();
        let last_name = input.last_name.trim().to_string();

        let rows = self
            .connection
            .execute(
                UPDATE_CONTACT,
                rusqlite::params![first_name, last_name, input.company, input.email, id],
            )
            .map_err(|e| StoreError::Database(e.to_string()))?;

        if rows == 0 {
            return Err(StoreError::NotFound(id));
        }

        let contact =
            self.contacts.iter_mut().find(|c| c.id == id).ok_or(StoreError::NotFound(id))?;

        contact.first_name = first_name;
        contact.last_name = last_name;
        contact.company = input.company.clone();
        contact.email = input.email.clone();

        Ok(contact.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> SqliteStore {
        SqliteStore::open(":memory:").expect("open in-memory db")
    }

    fn valid_input() -> NewContact {
        NewContact {
            first_name: String::from("Chuck"),
            last_name: String::from("Davis"),
            company: String::from("Dow Chem"),
            email: String::from("np@dowchem.com"),
            active: true,
        }
    }

    #[test]
    fn create_valid_input_succeeds() {
        let mut store = store();

        let contact = store.create(valid_input()).expect("valid input should be accepted");

        assert_eq!(contact.first_name, "Chuck");
        assert_eq!(store.list().len(), 1);
    }

    #[test]
    fn create_rejects_empty_name() {
        let mut store = store();
        let input = NewContact { first_name: String::from("   "), ..valid_input() };

        let result = store.create(input);

        assert!(
            matches!(result, Err(StoreError::InvalidInput(_))),
            "expected InvalidInput, got {result:?}"
        );
        assert!(store.list().is_empty(), "nothing should be stored on error");
    }

    #[test]
    fn create_assigns_unique_ids() {
        let mut store = store();

        let first = store.create(valid_input()).expect("first create");
        let second = store.create(valid_input()).expect("second create");

        assert_ne!(first.id, second.id);
    }

    #[test]
    fn delete_removes_existing_contact() {
        let mut store = store();
        let c = store.create(valid_input()).expect("create");

        store.delete(c.id).expect("delete should succeed");

        assert!(store.list().is_empty());
    }

    #[test]
    fn delete_missing_id_returns_not_found() {
        let mut store = store();

        let result = store.delete(42);

        assert!(matches!(result, Err(StoreError::NotFound(42))), "got {result:?}");
    }

    #[test]
    fn update_changes_fields_and_keeps_id() {
        let mut store = store();
        let original = store.create(valid_input()).expect("create");
        let edit = NewContact { company: String::from("BASF"), ..valid_input() };

        let updated = store.update(original.id, &edit).expect("update should succeed");

        assert_eq!(updated.id, original.id);
        assert_eq!(updated.company, "BASF");
        assert_eq!(store.list()[0].company, "BASF", "cached copy should change too");
    }

    #[test]
    fn update_rejects_invalid_input_and_leaves_contact_unchanged() {
        let mut store = store();
        let original = store.create(valid_input()).expect("create");
        let bad = NewContact { email: String::from("no-at-sign"), ..valid_input() };

        assert!(store.update(original.id, &bad).is_err());
        assert_eq!(store.list()[0].email, original.email);
    }

    #[test]
    fn update_missing_id_returns_not_found() {
        let mut store = store();

        let result = store.update(42, &valid_input());

        assert!(matches!(result, Err(StoreError::NotFound(42))), "got {result:?}");
    }

    #[test]
    fn data_persists_across_reopen() {
        let path = std::env::temp_dir().join("crm_test_persists_across_reopen.db");
        let path_str = path.to_str().expect("temp path should be valid UTF-8");
        let _ = std::fs::remove_file(&path); // clear leftovers from a failed earlier run

        {
            let mut store = SqliteStore::open(path_str).expect("first open");
            store.create(valid_input()).expect("create");
            store.update(1, &NewContact { company: String::from("BASF"), ..valid_input() })
                .expect("update");
        } // store dropped here, closing the connection

        let reopened = SqliteStore::open(path_str).expect("reopen");
        let contacts = reopened.list();

        assert_eq!(contacts.len(), 1, "contact should be reloaded from disk");
        assert_eq!(contacts[0].company, "BASF", "update should be persisted");

        drop(reopened);
        std::fs::remove_file(&path).expect("clean up test db");
    }
}