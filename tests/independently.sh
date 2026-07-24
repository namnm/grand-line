# test each module independently with minimal flags to make sure the exports and feature flags are correct

cargo test --no-default-features --features test_utils,sqlite --test err;

cargo test --no-default-features --features test_utils,sqlite --test model;
cargo test --no-default-features --features test_utils,sqlite --test relationship;
cargo test --no-default-features --features test_utils,sqlite --test soft_delete;

cargo test --no-default-features --features test_utils,sqlite,axum,authz,rand_utils --test authz;

cargo test --no-default-features --features test_utils,i18n --test i18n;
