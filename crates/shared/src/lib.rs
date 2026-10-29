#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("not found")]
    NotFound(u32),
    #[error("connection failed")]
    Database(String),
}

#[derive(Debug, Clone)]
pub struct Contact {
    pub id: u32,
    pub first_name: String,
    pub last_name: String,
    pub company: String,
    pub email: String,
    pub active: bool,
}

impl Contact {
    pub fn matches(&self, query: &str) -> bool {
        let q = query.trim().to_lowercase();

        self.first_name.to_lowercase().contains(&q)
            || self.last_name.to_lowercase().contains(&q)
            || self.company.to_lowercase().contains(&q)
            || self.email.to_lowercase().contains(&q)
    }
}

#[derive(Debug, Default, Clone)]
pub struct NewContact {
    pub first_name: String,
    pub last_name: String,
    pub company: String,
    pub email: String,
    pub active: bool,
}

pub trait ContactStore {
    fn list(&self) -> &[Contact];
    fn create(&mut self, input: NewContact) -> Result<Contact, StoreError>;
    fn delete(&mut self, id: u32) -> Result<(), StoreError>;
    fn update(&mut self, id: u32, input: &NewContact) -> Result<Contact, StoreError>;
}

pub fn validate_contact(input: &NewContact) -> Result<(), StoreError> {
    if input.first_name.trim().is_empty() {
        return Err(StoreError::InvalidInput("First name field was empty".to_string()));
    }
    if !input.email.contains("@") {
        return Err(StoreError::InvalidInput("Email is invalid".to_string()));
    }
    // any other validation here
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chuck() -> Contact {
        Contact {
            id: 1,
            first_name: String::from("Chuck"),
            last_name: String::from("Davis"),
            company: String::from("Dow Chem"),
            email: String::from("np@dowchem.com"),
            active: true,
        }
    }
    #[test]
    fn partial_company_query_matches() {
        let c: Contact = chuck();
        assert!(c.matches("dow"), "expected 'dow' to match company 'Dow Chem'");
    }

    fn dale() -> Contact {
        Contact {
            id: 2,
            first_name: String::from("Dale"),
            last_name: String::from("Brent"),
            company: String::from("BASF"),
            email: String::from("dale@basf.com"),
            active: true,
        }
    }

    #[test]
    fn partial_company_query_mismatches() {
        let d: Contact = dale();
        assert!(!d.matches("dow"), "expected 'dow' to not match company 'BASF'");
    }

    #[test]
    fn uppercase_query_matches() {
        assert!(
            chuck().matches("CHUCK"),
            "expected 'CHUCK' to match name 'Chuck' case-insensitively"
        );
    }

    #[test]
    fn email_query_matches() {
        assert!(chuck().matches("np@dow"), "expected 'np@dow' to match email 'np@dowchem.com'");
    }

    #[test]
    fn empty_query_matches_everything() {
        assert!(chuck().matches(""), "expected empty query to match");
    }

    #[test]
    fn whitespace_query_matches_everything() {
        assert!(chuck().matches("   "), "expected whitespace-only query to match");
    }

    #[test]
    fn padded_query_matches() {
        assert!(chuck().matches("  dow  "), "expected surrounding spaces to be ignored");
    }

    #[test]
    fn invalid_input_displays_message() {
        let err = StoreError::InvalidInput(String::from("name is required"));
        assert_eq!(err.to_string(), "invalid input: name is required");
    }
}
