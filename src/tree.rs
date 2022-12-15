trait TreeMap<K, V> {
    pub fn insert(&self, key: K, value: V);

    pub fn get(&self, key: K) -> Option<V>;

    pub fn delete(&self, key: K);
}