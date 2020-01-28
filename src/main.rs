use midir::{MidiInput, MidiOutput, MidiOutputConnection};
use ghakuf::messages::{MidiEvent, MidiEventBuilder};

// Display name for MIDI client
const CLIENT_NAME:      &'static str = "@selenologist buttons to MMC";
const INPUT_PORT_NAME:  &'static str = "@selenologist buttons to MMC input";
const OUTPUT_PORT_NAME: &'static str = "@selenologist buttons to MMC output";

// see https://en.wikipedia.org/wiki/MIDI_Machine_Control
const MMC_STOP:   &'static [u8] = &[0xf0, 0x7f, 0x00, 0x06, 0x01, 0xf7];
const MMC_PLAY:   &'static [u8] = &[0xf0, 0x7f, 0x00, 0x06, 0x02, 0xf7];
const MMC_REWIND: &'static [u8] = &[0xf0, 0x7f, 0x00, 0x06, 0x05, 0xf7]; // rewind
const MMC_RECE:   &'static [u8] = &[0xf0, 0x7f, 0x00, 0x06, 0x07, 0xf7]; // record exit
const MMC_RECP:   &'static [u8] = &[0xf0, 0x7f, 0x00, 0x06, 0x08, 0xf7]; // record pause
const MMC_PAUSE:  &'static [u8] = &[0xf0, 0x7f, 0x00, 0x06, 0x09, 0xf7];
// Ardour doesn't respond to Rewind, so instead send GOTO/LOCATE with all zeros
const MMC_GOTO0:  &'static [u8] = &[0xf0, 0x7f, 0x00, 0x06, 0x44,
                                    0x06, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf7];

const CC_ON_VALUE: u8 = 0x7f;
const CC_RECORD:   u8 = 50;
const CC_STOP:     u8 = 51;
const CC_PLAY:     u8 = 54;

struct HandlerData {
    last_time: u64,
    output: MidiOutputConnection,
    playing: bool // current playback state
}

fn handler(time: u64, midi_data: &[u8], data: &mut HandlerData) {
    // split time into whole seconds and the remaining microseconds
    let time_seconds = time / 1_000_000;
    let time_micros  = time % 1_000_000;

    let delta         = time.saturating_sub(data.last_time);
    let delta_seconds = delta / 1_000_000;
    let delta_micros  = delta % 1_000_000;
    
    data.last_time = time;
    
    // parse the midi message
    let mut mev_builder = MidiEventBuilder::new(midi_data[0]);
    for byte in &midi_data[1..midi_data.len()] {
        mev_builder.push(*byte);
    }
    
    let event: MidiEvent = mev_builder.build();

    // print the time and message
    println!("[{}.{:06}(+{}.{:06})] {:?}",
            time_seconds, time_micros, delta_seconds, delta_micros, event);

    if let MidiEvent::ControlChange{ ch: _ch, control, data: value } = event {
        match control {
            CC_RECORD => { // record update
                if value == CC_ON_VALUE { // record on
                    data.output.send(MMC_RECP).unwrap()
                }
                else { // record off
                    data.output.send(MMC_RECE).unwrap()
                }
            },
            CC_STOP if value == CC_ON_VALUE => { // stop button pressed
                if data.playing {
                    data.output.send(MMC_STOP).unwrap();
                    data.playing = false;
                }
                else {
                    // if we weren't playing when STOP was pressed, rewind
                    data.output.send(MMC_GOTO0).unwrap();
                }
            },
            CC_PLAY if value == CC_ON_VALUE => { // play button pressed
                if data.playing {
                    // if we were already playing when PLAY was pressed, pause instead
                    data.output.send(MMC_PAUSE).unwrap();
                    data.playing = false;
                }
                else {
                    data.output.send(MMC_PLAY).unwrap();
                    data.playing = true;
                }
            },
            _  => {}
        }
    }
}

fn main() {
    // Virtual Inputs and Outputs aren't supported on Windows.
    // So this program can't be compiled for Windows.
    use midir::os::unix::{VirtualInput, VirtualOutput};

    let output =
        MidiOutput::new(OUTPUT_PORT_NAME)
            .expect("Failed to open MIDI output")
            .create_virtual(OUTPUT_PORT_NAME)
            .expect("Failed to open MIDI output");
    
    let input =
        MidiInput::new(CLIENT_NAME)
            .expect("Failed to create MIDI input")
            .create_virtual(INPUT_PORT_NAME, handler,
                HandlerData{
                    last_time: 0,
                    playing: false,
                    output
                })
            .expect("Failed to create MIDI input port");

    // input events will now call the handler

    // block on a channel with no other writers in order to sleep forever
    // (Ctrl-C should still terminate the program as normal)
    let (tx, rx) = std::sync::mpsc::channel();
    rx.recv().unwrap();

    // this point is now unreachable. 'Use' tx so it won't be optimised out / dropped early.
    tx.send(()).unwrap();

    // close the input manually so that `input` is not 'unused' and therefore won't be optimised out
    input.close();
}
