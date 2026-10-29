use shared::{Contact, ContactStore, NewContact, StoreError, validate_contact};

#[derive(Debug, Default)]
pub struct MemoryStore {
    contacts: Vec<Contact>,
    next_id: u32,
}

impl ContactStore for MemoryStore {
    fn list(&self) -> &[Contact] {
        &self.contacts
    }

    fn create(&mut self, input: NewContact) -> Result<Contact, StoreError> {
        validate_contact(&input)?;

        let new = Contact {
            id: self.next_id,
            first_name: input.first_name.trim().to_string(),
            last_name: input.last_name.trim().to_string(),
            company: input.company,
            email: input.email,
            active: true,
        };
        self.next_id += 1;
        self.contacts.push(new.clone());
        Ok(new)
    }

    fn delete(&mut self, id: u32) -> Result<(), StoreError> {
        match self.contacts.iter().position(|c| c.id == id) {
            Some(index) => {
                self.contacts.remove(index);
                Ok(())
            }
            None => return Err(StoreError::NotFound(id)),
        }
    }

    fn update(&mut self, id: u32, input: &NewContact) -> Result<Contact, StoreError> {
        validate_contact(input)?;

        let contact =
            self.contacts.iter_mut().find(|c| c.id == id).ok_or(StoreError::NotFound(id))?;

        contact.first_name = input.first_name.trim().to_string();
        contact.last_name = input.last_name.trim().to_string();
        contact.company = input.company.clone();
        contact.email = input.email.clone();

        Ok(contact.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let mut store = MemoryStore::default();

        let contact = store.create(valid_input()).expect("valid input should be accepted");

        assert_eq!(contact.first_name, "Chuck");
        assert_eq!(store.list().len(), 1);
    }

    #[test]
    fn create_rejects_empty_name() {
        let mut store = MemoryStore::default();
        let input = NewContact { first_name: String::from("   "), ..valid_input() };

        let result = store.create(input);

        assert!(
            matches!(result, Err(StoreError::InvalidInput(_))),
            "expected InvalidInput, got {result:?}"
        );
        assert!(store.list().is_empty(), "nothing should be stored on error");
    }

    #[test]
    fn create_rejects_email_without_at() {
        let mut store = MemoryStore::default();
        let input = NewContact { email: String::from("np.dowchem.com"), ..valid_input() };

        assert!(store.create(input).is_err());
        assert!(store.list().is_empty());
    }

    #[test]
    fn create_assigns_unique_ids() {
        let mut store = MemoryStore::default();

        let first = store.create(valid_input()).expect("first create");
        let second = store.create(valid_input()).expect("second create");

        assert_ne!(first.id, second.id);
    }

    #[test]
    fn delete_removes_existing_contact() {
        let mut store = MemoryStore::default();
        let c = store.create(valid_input()).expect("create");

        store.delete(c.id).expect("delete should succeed");

        assert!(store.list().is_empty());
    }

    #[test]
    fn delete_missing_id_returns_not_found() {
        let mut store = MemoryStore::default();

        let result = store.delete(42);

        assert!(matches!(result, Err(StoreError::NotFound(42))), "got {result:?}");
    }

    #[test]
    fn update_changes_fields_and_keeps_id() {
        let mut store = MemoryStore::default();
        let original = store.create(valid_input()).expect("create");
        let edit = NewContact { company: String::from("BASF"), ..valid_input() };

        let updated = store.update(original.id, &edit).expect("update should succeed");

        assert_eq!(updated.id, original.id);
        assert_eq!(updated.company, "BASF");
        assert_eq!(store.list()[0].company, "BASF", "stored copy should change too");
    }

    #[test]
    fn update_rejects_invalid_input_and_leaves_contact_unchanged() {
        let mut store = MemoryStore::default();
        let original = store.create(valid_input()).expect("create");
        let bad = NewContact { email: String::from("no-at-sign"), ..valid_input() };

        assert!(store.update(original.id, &bad).is_err());
        assert_eq!(store.list()[0].email, original.email);
    }

    #[test]
    fn update_missing_id_returns_not_found() {
        let mut store = MemoryStore::default();

        let result = store.update(42, &valid_input());

        assert!(matches!(result, Err(StoreError::NotFound(42))), "got {result:?}");
    }
}
