use std::collections::HashMap;
use mockall::automock;
use std::sync::{Arc, Mutex};

#[automock]
pub trait StoreTrait {
    fn tell(&self, token: Box<str>) -> bool;
    fn ask(&self, token: &str) -> bool;
    fn get(&self, token: Box<str>) -> bool;
    fn nask(&self, token: &str) -> bool;
    fn clear_store(&self);
    fn print_store(&self);
}


/// **@summary** - The BachTStore struct is a store that keeps track of the number of occurrences of a token
///
/// Using HashMap, see [reference](https://doc.rust-lang.org/std/collections/struct.HashMap.html).
pub(crate) struct Store {
    the_store: Arc<Mutex<HashMap<Box<str>, u32>>>
}


impl StoreTrait for Store {

    /// **@summary** - It adds one occurrence of the token to the store
    ///
    /// **@param** token: &str - The token to add to the store
    ///
    /// **@returns** - Always true
    ///
    /// Nbr of occurrences of the token is encoded using u32. So it ignores incrementation if it reaches the u32's max value.
    /// See [reference](https://doc.rust-lang.org/std/collections/hash_map/enum.Entry.html).
    fn tell(&self, token: Box<str>) -> bool {
        self.the_store.lock().unwrap().entry(token).and_modify(|nbr_occurrence| {
            *nbr_occurrence = Self::safe_inc(*nbr_occurrence);
        }).or_insert(1);
        true
    }

    /// **@summary** - It checks if the token is in the store
    ///
    /// **@param** token: &str - The token to check in the store
    ///
    /// **@returns** - true if the token is in the store, false otherwise
    fn ask(&self, token: &str) -> bool {
        let unlock_store = self.the_store.lock().unwrap();
        if !unlock_store.contains_key(token) {
            false
        } else {
            unlock_store.get(token).unwrap() > &0
        }
    }

    /// **@summary** - It checks if the token is in the store and removes one occurrence of it
    ///
    /// **@param** token: &str - The token to check in the store
    ///
    /// **@returns** - true if the token is in the store, false otherwise
    fn get(&self, token: Box<str>) -> bool {
        let mut res = false;

        self.the_store.lock().unwrap().entry(token).and_modify(|nbr_occurrence| {
            if *nbr_occurrence > 0 {
                *nbr_occurrence -= 1;
                res = true;
            }
        });
        res
    }

    /// **@summary** - It checks if the token is absent from the store
    ///
    /// **@param** token: &str - The token to check in the store
    ///
    /// **@returns** - true if the token is absent from the store, false if it is present
    fn nask(&self, token: &str) -> bool {
        let unlock_store = self.the_store.lock().unwrap();
        if !unlock_store.contains_key(token) {
            true
        } else {
            unlock_store.get(token).unwrap() <= &0
        }
    }

    /// **@summary** - It clears the store
    fn clear_store(&self) {
        self.the_store.lock().unwrap().clear();
    }

    fn print_store(&self) {
        print!("=== Store ===\n");
        for (key, value) in self.the_store.lock().unwrap().iter() {
            println!("{}({})", key, value);
        }
        print!("\n");
    }
}

impl Store {
    /// Create a new Store
    pub(crate) fn new() -> Store {
        Store {
            the_store: Arc::from(Mutex::new(HashMap::new()))
        }
    }

    /// Create a new store with predefined data
    pub(crate) fn new_with_data(data: HashMap<Box<str>, u32>) -> Store {
        Store {
            the_store: Arc::from(Mutex::new(data))
        }
    }

    /// **@summary** - It increments a number by one safely
    ///
    /// **@param** nbr: u32 - The number to increment
    ///
    /// **@returns** - The incremented number if it is less than u32's max value, the number itself otherwise
    fn safe_inc(nbr: u32) -> u32 {
        if nbr < u32::MAX {
            nbr + 1
        } else {
            nbr
        }
    }
}


/// ===============
/// |    TESTS    |
/// ===============

#[cfg(test)]
mod tests {
    use super::*;

    fn get_data(store: &Store) -> HashMap<Box<str>, u32> {
        store.the_store.lock().unwrap().clone()
    }

    // tell section
    #[test]
    fn the_store_should_be_able_to_tell_whatever_its_data_state() {
        let clear_store = Store::new(); // empty store
        let used_store = Store::new_with_data(
            HashMap::from([("token".into(), 1)])
        );

        assert!(clear_store.tell("token".into()));
        assert!(used_store.tell("token".into()));
    }

    #[test]
    fn the_store_should_add_a_new_token_when_tell_if_doesnt_exists() {
        let store = Store::new();
        let res = store.tell("token".into());
        assert!(res);
        assert!(get_data(&store).contains_key("token"));
    }

    #[test]
    fn the_store_should_increment_token_when_tell_if_it_exists() {
        let store = Store::new_with_data(HashMap::from([("token".into(), 1)]));
        assert!(store.tell("token".into()));
        assert_eq!(get_data(&store).get("token").unwrap(), &2);
    }

    #[test]
    fn the_store_should_not_allow_max_occurrence_overflow() {
        let store = Store::new_with_data(HashMap::from([("token".into(), u32::MAX)]));
        let res = store.tell("token".into());
        assert!(res);
        assert_eq!(get_data(&store).get("token").unwrap(), &u32::MAX);
    }

    // ask section

    #[test]
    fn the_store_should_be_able_to_ask_if_one_or_more_occurrence_of_token() {
        let store = Store::new_with_data(HashMap::from([("token".into(), 1)]));
        assert!(store.ask("token"));
        assert_eq!(get_data(&store).get("token").unwrap(), &1);
    }

    #[test]
    fn the_store_should_not_be_able_to_ask_if_zero_occurrence_of_token() {
        let store = Store::new_with_data(HashMap::from([("token".into(), 0)]));
        assert!(!store.ask("token"));
        assert_eq!(get_data(&store).get("token").unwrap(), &0);
    }

    #[test]
    fn the_store_should_not_be_able_to_ask_if_no_occurrence_of_token() {
        let store = Store::new();
        assert!(!store.ask("token"));
        assert!(!get_data(&store).contains_key("token"));
    }

    // get section

    #[test]
    fn the_store_should_be_able_to_get_one_occurrence_of_token() {
        let store = Store::new_with_data(HashMap::from([("token".into(), 1)]));
        assert!(store.get("token".into()));
        assert_eq!(store.the_store.lock().unwrap().get("token").unwrap(), &0);
    }

    #[test]
    fn the_store_should_not_be_able_to_get_if_zero_occurrence_of_token() {
        let store = Store::new_with_data(HashMap::from([("token".into(), 0)]));
        assert!(!store.get("token".into()));
        assert_eq!(get_data(&store).get("token").unwrap(), &0);
    }

    #[test]
    fn the_store_should_not_be_able_to_get_if_no_occurrence_of_token() {
        let store = Store::new();
        let res = store.get("token".into());
        assert!(!res);
    }

    // nask section

    #[test]
    fn the_store_should_be_able_to_nask_if_zero_occurrence_of_token() {
        let store = Store::new_with_data(
            HashMap::from([("token".into(), 0)])
        );
        assert!(store.nask("token"));
        assert_eq!(get_data(&store).get("token").unwrap(), &0);
    }

    #[test]
    fn the_store_should_be_able_to_nask_if_no_occurrence_of_token() {
        let store = Store::new();
        let res = store.nask("token");
        assert!(res);
        assert!(!get_data(&store).contains_key("token"));
    }

    #[test]
    fn the_store_should_not_be_able_to_nask_if_one_or_more_occurrence_of_token() {
        let store = Store::new_with_data(HashMap::from([("token".into(), 1)]));
        let res = store.nask("token");
        assert!(!res);
        assert_eq!(get_data(&store).get("token").unwrap(), &1);
    }

    // Clear_store section

    #[test]
    fn the_store_should_be_able_to_clear_its_data() {
        let store = Store::new_with_data(HashMap::from([("token".into(), 1)]));
        store.clear_store();
        assert!(get_data(&store).is_empty());
    }

    // Print_store section

    #[test]
    fn the_store_should_be_able_to_print_its_data() {
        let store = Store::new_with_data(HashMap::from([
            ("tameImpala".into(), 5),
            ("daftPunk".into(), u32::MAX),
            ("gorillaz".into(), 0)
        ]));
        store.print_store();
    }
}