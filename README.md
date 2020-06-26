ac-trainer is a Rust program that makes you _really_ good at AssaultCube.

It is an "external" trainer, meaning that it interacts with AssaultCube through Windows' ReadProcessMemory and WriteProcessMemory API, rather than, say, injecting a DLL and hijacking/spawning a thread in AssaultCube.

Currently, it will:

* Print out each player's name, health, armor and position every second
* If Caps Lock is active it will aim at the closest living player
* Godmode

This project was was undertaken in order to get better at Rust, to learn a little bit about game hacking, and to do _something_ with Windows for the first time in a long time. To maximize learning, it does not pull in any external dependencies.
