pub mod word_tree;

#[cfg(test)]
fn init_logger() {
    use log::LevelFilter;

    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(LevelFilter::max())
        .try_init();
}
