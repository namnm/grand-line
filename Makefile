fmt:
	@cargo +nightly fmt --all \
	&& doctoc --loglevel warn --github README.md \
	&& dprint fmt --log-level warn;

check:
	@make fmt \
	&& cargo clippy --workspace --all-targets --all-features --fix --allow-dirty;

test:
	@make fmt \
	&& cargo test --features test_utils;

test_mysql:
	@make fmt \
	&& cargo test --no-default-features --features test_utils,default_without_db,mysql;

test_sqlite:
	@make fmt \
	&& cargo test --no-default-features --features test_utils,default_without_db,sqlite;

test_each:
	@make fmt \
	&& bash ./tests/independently.sh;

update:
	@make fmt \
	&& cargo update --dry-run \
	&& cargo install-update -a \
	&& cargo upgrade --incompatible \
	&& dprint upgrade \
	&& dprint config update;

imagemin:
	@export EXT="png|jpg|gif|ico" \
	&& make git-ls \
	| xargs -L1 bash -c 'imagemin $$0 --out-dir $$(dirname $$0)';
git-ls:
	@bash -c 'comm -3 <(git ls-files) <(git ls-files -d)' \
	| egrep -h '\.($(EXT))$$';

# missing trailing comma
# [^,\s.*][\s\n]+[\)\]]
# ,[)\]]
