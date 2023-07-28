brew services stop postgresql@15
rm /tmp/pgml_bg_worker.sock
kill -9 $(pgrep pgml_bgworker)
cargo pgrx install
cd bgworker_example && cargo pgrx install
brew services start postgresql@15
