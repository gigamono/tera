#[macro_export]
macro_rules! set {
    // Match zero or more comma delimited items
    ( $( $x:expr ),* $(,)? ) => {
        {
            use hashbrown::HashSet;

            // Create a mutable HashSet
            let mut temp_set = HashSet::new();
            $(
                // Insert each item matched into the HashSet
                temp_set.insert($x);
            )*

            temp_set
        }
    };
}
