# viewd
An image viewer server and client for viewing images on a remote box
(presumably with monitor connected). It uses sdl2 for image display
and TCP for network communication. All connections are encrypted and
authorized by TLS. Network protocol borrows heavily from tokio's
[mini-redis](https://github.com/tokio-rs/mini-redis) project.

Only tested on linux.

## setup

Networking is setup to use TLS by default, so you will first need to
configure certificates. [This
script](https://github.com/rustls/tokio-rustls/blob/main/scripts/generate-certificate.sh)
can generate them for you if you like. Then make sure your
configuration files point to the correct keys and certificates.

## usage

Requires `--path` parameter to declare filesystem location from which
application will load images.

on the display box:

	cargo run --bin viewd-server -- --path ~/dir/photos/

on the client:

	cargo run --bin viewd-tui

### commands

Currently supported commands are

	* `->` (arrow right) next image
    * `<-` (arrow left) previous image
	* `f`  fullscreen
	* `r`  rotate
	* `p`  pageant mode (automatically scroll through the images)
    * `q`  quit (the client)

`pageant` make the image advance automatically every second. The rest
should be self explanitory.

### viewd-cli

In a addition to the TUI, there is also a cli. You can get its usage
by passing the `--help` option.

	cargo run --bin viewd-cli -- --help

## Todo

  * lots of stuff
  * ???

## system dependencies

You need sdl libraries on your OS. Mileage may vary depending on system,
but on debian-like apt can obtain them for you:

	sudo apt-get install libsdl2-image-2.0-0

## display

You may need to export your display. `:1` may or may not be correct
depending on your system.

	export DISPLAY=:1

## development

You will need development libraries for builds to complete.

	sudo apt-get install libsdl2-image-dev
