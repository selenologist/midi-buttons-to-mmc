# midi-buttons-to-mmc

I have an Arturia Minibrute 2 synthesizer, that has Record, Stop, and Play buttons. When these buttons are pushed, a MIDI CC is generated to reflect their state.

This program converts these MIDI CC messages into MIDI Machine Control (MMC) messages, so that I can control DAWs like Ardour using these buttons. Doubles as a MIDI monitor.

Note that standards-compliance was not a goal, only controlling Ardour in a way that was useful to me. YMMV.

TIP: To allow using the record button without messing up the Minibrute 2's internal sequencer as much, use the Sync button to set the clock source to something not connected to anything (e.g. the patchbay clock jack), so that the internal sequencer will never update.

# Portability

Does not work on Windows as it creates Virtual Input and Output ports and waits for other programs to connect to it rather than connecting directly to an output. As Windows doesn't support this, this program cannot work. It will fail to compile.
