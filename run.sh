set -e

cargo build --release
cd target/release
cp libminebot.dylib libminebot.so
cp ../../bot.py .
python bot.py
