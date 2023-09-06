# viewd
A image veiwer server and client for viewing images on a remote box (presumably with monitor connected). Probably no for everyone, but it is useful to me. It uses sdl2 for image dispaly and TCP for network commnunication. Network protocol is a subset of redis communication protocol.

Only tested on linux.

## Todo

  * TLS (or some form of mutual verification and validation)
  * terminal interface
  * ???

## usage

Accepts a directory containing images as input. It *should* filter out
non-image files. No recursion is done into sub-directories.

on the display box:

	viewd-server --path ~/dir/photos/

on the client:

	viewd-cli next

Where `next` is any supported command.

### commands

Currently supported commands are

  * `next`
  * `prev`
  * `fullscreen`
  * `rotate`
  * `pageant`

`pageant` make the image advance automatically every second. The rest
should be self explanitory.

## dependencies

You need sdl libraries on your OS. Milage may vary depending on sytem, but on debian-like apt can obtain them for you:

	sudo apt-get install libsdl2-image-2.0-0

## display

You may need to export your display. `:1` may or may not be correct
depending on your system.

	export DISPLAY=:1

## development

You will need development libraries for builds to complete.

	sudo apt-get install libsdl2-image-dev
