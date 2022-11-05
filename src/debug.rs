// # Put a statement to measure its execution time
#[macro_export]
macro_rules! perf {
    ($statement: stmt) => {
        let start = crate::Instant::now();
        $statement
        let end = start.elapsed();

        crate::debug!("{:?} took {:?}", stringify!($statement), end);
    };
}