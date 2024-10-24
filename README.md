# Simple twitch chat viewer
A simple twitch chat viewer, this kind does **not** need a twitch OAuth token.

Aside from that, this has some nice stuff, like colored names, & a few twitch badges.
You can also backup the IRC messages to a file, then replay those messages later.
However, this doesn't process emotes & has no intention to.
There are a few other things I'd like to implement but this is suitable for now.

This program is built on the [twitch-irc] library, all credit should go to them.
Seriously, this program is basically a wrapper around this library.

### License
    Copyright (C) 2024  Josiah Baldwin

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.

[twitch-irc]: https://github.com/robotty/twitch-irc-rs
