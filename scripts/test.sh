set -e

# TODO merge test and build for features 
echo "\n>\n fmt && test && build \n>\n"
cargo fmt && cargo test && cargo build

echo "\n>\n test --features=api \n>\n"
cargo test --features=api 
echo "\n>\n test --features=cloud \n>\n"
cargo test --features=cloud
echo "\n>\n test --features=python \n>\n"
cargo test --features=python
echo "\n>\n test --features=napi \n>\n"
cargo test --features=napi
echo "\n>\n test --features=async \n>\n"
cargo test --features=async
echo "\n>\n test --features=wasm \n>\n"
wasm-pack test --node --features=wasm && wasm-pack test --node --features=insecure,wasm
