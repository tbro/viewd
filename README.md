# viewd
A image viewer server and client for viewing images on a remote box
(presumably with monitor connected). Probably no for everyone, but it
is useful to me. It uses sdl2 for image display and TCP for network
communication. Network protocol is a subset of redis communication
protocol.

Only tested on linux.

## Todo

  * lots of stuff
  * ???

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

## FAQ

### Some times I have to hit `next` twice to advance
This will mostly happen in a directory where you have non-image
files. `next` will silently fail when sdl2 can't load a file. So you
have to advance until you get to a supported image file. This also
occurs when changing direction (from `next` to `prev` or `prev` to
`next`) due to how the cursor works.

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
