use std::collections::HashMap;
/// **@summary** - The BachTStore struct is a store that keeps track of the number of occurrences of a token
///
/// Using HashMap, see [reference](https://doc.rust-lang.org/std/collections/struct.HashMap.html).
pub struct BachTStore<'a> {
    the_store: HashMap<&'a str,u32>
}

impl<'a> BachTStore<'a> {
    /// Create a new BachTStore
    fn new() -> BachTStore<'a> {
        BachTStore {
            the_store: HashMap::new()
        }
    }

    /// **@summary** - It adds one occurrence of the token to the store
    ///
    /// **@param** token: &str - The token to add to the store
    ///
    /// **@returns** - Always true
    ///
    /// Nbr of occurrences of the token is encoded using u32. So it ignores incrementation if it reaches the u32's max value.
    /// See [reference](https://doc.rust-lang.org/std/collections/hash_map/enum.Entry.html).
    fn tell(&mut self, token: &'a str) -> bool {
        self.the_store.entry(token).and_modify(|nbr_occurrence| {
            if *nbr_occurrence < u32::MAX {
                *nbr_occurrence += 1;
            }
        }).or_insert(1);
        true
    }

    /// **@summary** - It checks if the token is in the store
    ///
    /// **@param** token: &str - The token to check in the store
    ///
    /// **@returns** - true if the token is in the store, false otherwise
    fn ask(&mut self, token: &'a str) -> bool {
        if !self.the_store.contains_key(token) {
            false
        } else {
            self.the_store.get(token).unwrap() > &0
        }
    }

    /// **@summary** - It checks if the token is in the store and removes one occurrence of it
    ///
    /// **@param** token: &str - The token to check in the store
    ///
    /// **@returns** - true if the token is in the store, false otherwise
    fn get(&mut self, token: &'a str) -> bool {
        let mut res = false;
        self.the_store.entry(token).and_modify(|nbr_occurrence| {
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
    fn nask(&mut self, token: &str) -> bool {
        if !self.the_store.contains_key(token) {
            true
        } else {
            self.the_store.get(token).unwrap() <= &0
        }
    }

    /// **@summary** - It clears the store
    fn clear_store(&mut self) {
        self.the_store.clear();
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    // tell section
    #[test]
    fn the_store_should_be_able_to_tell_whatever_its_data_state() {
        let mut store = BachTStore::new(); // empty store
        let res_without_data = store.tell("token");

        store.the_store = HashMap::from([("token", 1)]); // store with data
        let res_with_data = store.tell("token");
        assert!(res_without_data && res_with_data);
    }

    #[test]
    fn the_store_should_add_a_new_token_when_tell_if_doesnt_exists() {
        let mut store = BachTStore::new();
        let res = store.tell("token");
        assert!(res && store.the_store.contains_key("token"));
    }

    #[test]
    fn the_store_should_increment_token_when_tell_if_it_exists() {
        let mut store = BachTStore { // instanced with data
            the_store: HashMap::from([("token", 1)])
        };
        let res = store.tell("token");
        assert!(res && store.the_store.get("token").unwrap() == &2);
    }

    #[test]
    fn the_store_should_not_allow_max_occurrence_overflow() {
        let mut store = BachTStore {
            the_store: HashMap::from([("token", u32::MAX)])
        };
        let res = store.tell("token");
        assert!(res && store.the_store.get("token").unwrap() == &u32::MAX);
    }

    // ask section

    #[test]
    fn the_store_should_be_able_to_ask_if_one_or_more_occurrence_of_token() {
        let mut store = BachTStore {
            the_store: HashMap::from([("token", 1)])
        };
        let res = store.ask("token");
        assert!(res && store.the_store.get("token").unwrap() == &1);
    }

    #[test]
    fn the_store_should_not_be_able_to_ask_if_zero_occurrence_of_token() {
        let mut store = BachTStore {
            the_store: HashMap::from([("token", 0)])
        };
        let res = store.ask("token");
        assert!(!res && store.the_store.get("token").unwrap() == &0);
    }

    #[test]
    fn the_store_should_not_be_able_to_ask_if_no_occurrence_of_token() {
        let mut store = BachTStore::new();
        let res = store.ask("token");
        assert!(!res && !store.the_store.contains_key("token"));
    }

    // get section

    #[test]
    fn the_store_should_be_able_to_get_one_occurrence_of_token() {
        let mut store = BachTStore {
            the_store: HashMap::from([("token", 1)])
        };
        let res = store.get("token");
        assert!(res && store.the_store.get("token").unwrap() == &0);
    }

    #[test]
    fn the_store_should_not_be_able_to_get_if_zero_occurrence_of_token() {
        let mut store = BachTStore {
            the_store: HashMap::from([("token", 0)])
        };
        let res = store.get("token");
        assert!(!res && store.the_store.get("token").unwrap() == &0);
    }

    #[test]
    fn the_store_should_not_be_able_to_get_if_no_occurrence_of_token() {
        let mut store = BachTStore::new();
        let res = store.get("token");
        assert!(!res);
    }

    // nask section

    #[test]
    fn the_store_should_be_able_to_nask_if_zero_occurrence_of_token() {
        let mut store = BachTStore {
            the_store: HashMap::from([("token", 0)])
        };
        let res = store.nask("token");
        assert!(res && store.the_store.get("token").unwrap() == &0);
    }

    #[test]
    fn the_store_should_be_able_to_nask_if_no_occurrence_of_token() {
        let mut store = BachTStore::new();
        let res = store.nask("token");
        assert!(res && !store.the_store.contains_key("token"));
    }

    #[test]
    fn the_store_should_not_be_able_to_nask_if_one_or_more_occurrence_of_token() {
        let mut store = BachTStore {
            the_store: HashMap::from([("token", 1)])
        };
        let res = store.nask("token");
        assert!(!res && store.the_store.get("token").unwrap() == &1);
    }

    // Clear_store section

    #[test] //"This store" should "be able to clear its data"
    fn the_store_should_be_able_to_clear_its_data() {
        let mut store = BachTStore {
            the_store: HashMap::from([("token", 1)])
        };
        store.clear_store();
        assert!(store.the_store.is_empty());
    }
}