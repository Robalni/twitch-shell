# Twitch shell

This is a shell that makes it possible to access the Twitch API by
typing commands.

The license is CC0. You find the license text in the file LICENSE.

## Features

* Run commands non-interactively from outside of the shell, like: `$
  ./twitch-shell status = Making a cool program` to change your stream
  title

* Can handle an unlimited number of users. Switch user like this:
  `user=robalni`

* One letter shortcuts for common commands (like `f` for `following`)

* Search, follow, unfollow, watch streams and more

## Example usage

```
$ ./twitch-shell
twitch> search programming
HappyLittleRat playing Creative
  [Silent C++] Workbench #programming #C++ #gamedev 
dylan_landry playing Creative
  Relaxing Programming: Help, chill, learn, and relax. Day 11
[...]
twitch> login
[...]
robalni@twitch> user
robalni (id:94736345, has_oauth:yes)
robalni@twitch> status
Robalni playing Creative
  Doing something boring
robalni@twitch> status = Programming a Twitch shell
Robalni playing Creative
  Programming a Twitch shell
```
