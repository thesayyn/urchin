#!/usr/bin/env bash

cargo build --release 

cp target/release/fides ./Notify.app/Contents/MacOS/notify
cp -r Notify.app /Applications/

open -a Notify --args --title "Build completed!" --message test --subtitle hello