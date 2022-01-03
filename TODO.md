Finished 2.4

Some stuff I want to add that's not covered
- Unit tests of the radial FOV logic stuff?
- Better management of the worldpos / transform / layers situation; maybe use bundles?

Then on to 2.5

Start thinking about dialogue boxes (with choices and callbacks) since that's one big thing I think I never figured out
in previous attempts (the choices could do ALMOST ANYTHING in the world)

Start thinking about things that can happen when something is first seen / touched / whatever
Basically you can trigger a "seen" event for each seeable entity (or room, or whatever) and have some downstream
    system(s) handle this.

Logging is captured in 2.7 so that's a good time to learn about bevy ui components
Main menu is covered in 2.10 so that's a good time to learn about bevy game states
    although I'm not sure I actually care about save/load, if it's easy, I'd like to be able to do it
    (bevy should make this easy)