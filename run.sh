set -e

cargo build
cd target/debug
cp libminebot.dylib libminebot.so
cp ../../bot.py .
python bot.py
