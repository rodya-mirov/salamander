What It Is
----------

I'm going through bracket's roguelike tutorial [link](https://bfnightly.bracketproductions.com/rustbook/), in
rust + bevy, with lots of modifications.

I've done it before but it's fun to go through and do again. You learn more through repetition, finding new ways to do
old things, and so on. I'm particularly happy with these so far:

* the FOV algorithm I used is extremely elegant this time. It's a modification of the "shadow line" algorithm
    from Bob Nystrom's blog [link](http://journal.stuffwithstuff.com/2015/09/07/what-the-hero-sees/) but without
    octants, which I find sort of gross and buggy. Instead it uses a full circle flowing out from the player!

* I broke down and used tilesets from OpenGameArt this time and it's really a nice improvement. Using the images
    definitely restricts how you imagine the game world is -- you can't make it ambiguous about what the setting _is_
    when the images are quite specific -- but it sure looks prettier this way.

I haven't decided how far I'm going to go with this project. I don't know that I care to make a "real game" but there
are a few things I'd like to get figured out this time that I didn't manage last time:

* A bigger, semi-scripted world generated from raws
* Dialogue boxes with "arbitrary callbacks" from choices (open door, buy thing, execute the prisoner, whatever)
  in a way where the code doesn't completely suck.