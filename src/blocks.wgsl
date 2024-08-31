#define_import_path blocks

#import direction::{up, down, north, east}

const grass = 1u;
const dirt = 2u;
const stone = 3u;

fn texture_layer(block_id: u32, direction: u32) -> u32 {
    switch block_id {
        case grass: {
            switch direction {
                case up: { return 0u; }
                case down: { return 2u; }
                default: { return 1u; }
            }
        }
        case dirt: { return 2u; }
        case stone: { return 3u; }
        default: { return u32(-1i); }
    }
}

