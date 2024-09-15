use embassy_sync::mutex::Mutex;
use pumpedli::display::lcd199::Position;
use pumpedli::{control, scaling::Scaling};

use super::{analog, digital};

pub static LOOPS: [control::Loop; 16] = [
    control::Loop {
        adc_input: &analog::INPUTS[0],
        mux_output: &digital::OUTPUTS[0],
        lcd_position: Some(Position::Top),
        scaling: Mutex::new(Scaling::TYPE0_3V3),
        program: Mutex::new(None),
    },
    control::Loop {
        adc_input: &analog::INPUTS[1],
        mux_output: &digital::OUTPUTS[1],
        lcd_position: Some(Position::TopLeft),
        scaling: Mutex::new(Scaling::TYPE0_3V3),
        program: Mutex::new(None),
    },
    control::Loop {
        adc_input: &analog::INPUTS[2],
        mux_output: &digital::OUTPUTS[2],
        lcd_position: Some(Position::TopRight),
        scaling: Mutex::new(Scaling::TYPE0_3V3),
        program: Mutex::new(None),
    },
    control::Loop {
        adc_input: &analog::INPUTS[3],
        mux_output: &digital::OUTPUTS[3],
        lcd_position: Some(Position::CenterLeft),
        scaling: Mutex::new(Scaling::TYPE0_3V3),
        program: Mutex::new(None),
    },
    control::Loop {
        adc_input: &analog::INPUTS[4],
        mux_output: &digital::OUTPUTS[4],
        lcd_position: Some(Position::Center),
        scaling: Mutex::new(Scaling::TYPE0_3V3),
        program: Mutex::new(None),
    },
    control::Loop {
        adc_input: &analog::INPUTS[5],
        mux_output: &digital::OUTPUTS[5],
        lcd_position: Some(Position::CenterRight),
        scaling: Mutex::new(Scaling::TYPE0_3V3),
        program: Mutex::new(None),
    },
    control::Loop {
        adc_input: &analog::INPUTS[6],
        mux_output: &digital::OUTPUTS[6],
        lcd_position: Some(Position::BottomLeft),
        scaling: Mutex::new(Scaling::TYPE0_3V3),
        program: Mutex::new(None),
    },
    control::Loop {
        adc_input: &analog::INPUTS[7],
        mux_output: &digital::OUTPUTS[7],
        lcd_position: Some(Position::BottomRight),
        scaling: Mutex::new(Scaling::TYPE0_3V3),
        program: Mutex::new(None),
    },
    control::Loop {
        adc_input: &analog::INPUTS[8],
        mux_output: &digital::OUTPUTS[8],
        lcd_position: Some(Position::Bottom),
        scaling: Mutex::new(Scaling::TYPE0_3V3),
        program: Mutex::new(None),
    },
    control::Loop {
        adc_input: &analog::INPUTS[9],
        mux_output: &digital::OUTPUTS[9],
        lcd_position: None,
        scaling: Mutex::new(Scaling::TYPE0_3V3),
        program: Mutex::new(None),
    },
    control::Loop {
        adc_input: &analog::INPUTS[10],
        mux_output: &digital::OUTPUTS[10],
        lcd_position: None,
        scaling: Mutex::new(Scaling::TYPE0_3V3),
        program: Mutex::new(None),
    },
    control::Loop {
        adc_input: &analog::INPUTS[11],
        mux_output: &digital::OUTPUTS[11],
        lcd_position: None,
        scaling: Mutex::new(Scaling::TYPE0_3V3),
        program: Mutex::new(None),
    },
    control::Loop {
        adc_input: &analog::INPUTS[12],
        mux_output: &digital::OUTPUTS[12],
        lcd_position: None,
        scaling: Mutex::new(Scaling::TYPE0_3V3),
        program: Mutex::new(None),
    },
    control::Loop {
        adc_input: &analog::INPUTS[13],
        mux_output: &digital::OUTPUTS[13],
        lcd_position: None,
        scaling: Mutex::new(Scaling::TYPE0_3V3),
        program: Mutex::new(None),
    },
    control::Loop {
        adc_input: &analog::INPUTS[14],
        mux_output: &digital::OUTPUTS[14],
        lcd_position: None,
        scaling: Mutex::new(Scaling::TYPE0_3V3),
        program: Mutex::new(None),
    },
    control::Loop {
        adc_input: &analog::INPUTS[15],
        mux_output: &digital::OUTPUTS[15],
        lcd_position: None,
        scaling: Mutex::new(Scaling::TYPE0_3V3),
        program: Mutex::new(None),
    },
];
