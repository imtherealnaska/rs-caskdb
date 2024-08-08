pub trait Store {
    fn get(&self, key: &String) -> Option<&String>;
    fn set(&mut self, key: String, value: String);
    fn close(&self) -> bool;
}
