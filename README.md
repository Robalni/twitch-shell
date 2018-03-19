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

* Tab completion for names you have seen.

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

## Commands and variables

| Command               | Description
| --------------------- | ------------
| ?                     | Prints help text
| api [path]            | Explore the api
| exit                  | Exits the shell
| f                     | Alias for following
| follow <channel...>   | Follows the channel(s)
| unfollow <channel...> | Unfollows the channel(s)
| following             | Shows online streams you follow
| help                  | Prints help text
| login                 | Logs in to Twitch
| s [str [page]]        | Alias for search or status if no arguments
| search <str> [page]   | Searches for streams
| status [channel...]   | Shows info about channels (or your channel)
| streams [channel...]  | Shows info about streams (or your stream) if online
| time [channel...]     | Shows for how long the channels have been streaming
| user                  | Prints information about current user
| vods [channel [page]] | Shows a list of videos from the channel
| w <channel>           | Alias for watch
| watch <channel>       | Watch a stream (using mpv)

| Variable              | Description
|-----------------------|-------------
| game                  | The game you are playing
| status                | Status/title of the stream
| user                  | Name of current user
