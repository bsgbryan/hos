# Welcome!

Hello there and welcome to HOS, thank you for stopping by!

Head over to the [Wiki](https://github.com/bsgbryan/hos/wiki) for info and/or if you're interested in contributing!

# Status

HOS is _very_ early in development; I haven't tested it on read hardware yet - it's been QEMU all the way baby!

If you want to check HOS out, by all means, clone the repo and `make qemu` 🤘🏻

# Setup

### Steps to get `make miniterm` working on macOS Sequoia (v15)

Execute the following from the project root

1. `brew install rbenv ruby-build`
1. `rbenv init`
1. `rbenv install 3.1.7` (_this matches the version specified in .ruby-version_)
1. `bundle config build.serialport -- --with-cflags=-Wno-int-conversion` (_to work around a change introduced in Sequoia: [source](https://github.com/hparra/ruby-serialport/issues/74#issuecomment-2368049997)_)
1. `bundle install`

`make miniterm` should now work!

# Credit

HOS is being built on the great work of the [RusPiRo](https://github.com/RusPiRo) and [Rust Raspberry Pi OS Tutorials](https://github.com/rust-embedded/rust-raspberrypi-OS-tutorials) projects; thank you so much!
