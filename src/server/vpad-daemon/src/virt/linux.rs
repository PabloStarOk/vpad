use std::io::Result;

use evdev_rs::{
    AbsInfo, DeviceWrapper, EnableCodeData, InputEvent, TimeVal, UInputDevice, UninitDevice,
    enums::{BusType, EV_ABS, EV_FF, EV_KEY, EV_SYN, EventCode},
};

use crate::{
    transport::models::{
        Axis, Button, InputSignal,
        InputType::{self},
        Trigger,
    },
    virt::constants,
};

pub struct LinuxVirtualGamepad {
    virtual_dev: UInputDevice,
    dpad_state: DpadState,
}

struct DpadState {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

impl LinuxVirtualGamepad {
    const ZERO_TIME_VAL: TimeVal = TimeVal {
        tv_sec: 0,
        tv_usec: 0,
    };
    const SYN_REPORT_EVENT: InputEvent = InputEvent::new(
        &Self::ZERO_TIME_VAL,
        &EventCode::EV_SYN(EV_SYN::SYN_REPORT),
        0,
    );

    pub fn new() -> Result<Self> {
        let unint_dev =
            evdev_rs::UninitDevice::new().expect("Could not initialize virtual gamepad.");
        unint_dev.set_name("XBOX Virtual Controller");
        unint_dev.set_bustype(BusType::BUS_VIRTUAL as u16);
        unint_dev.set_vendor_id(constants::XBOX_VID);
        unint_dev.set_product_id(constants::XBOX_PID);
        Self::enable_events(&unint_dev)?;
        let virtual_dev = UInputDevice::create_from_device(&unint_dev)?;
        Ok(LinuxVirtualGamepad {
            virtual_dev,
            dpad_state: DpadState {
                up: false,
                down: false,
                left: false,
                right: false,
            },
        })
    }

    pub fn report_input(&mut self, signal: InputSignal) -> Result<()> {
        let event_code = match signal.itype {
            InputType::Button(button) => match button {
                Button::South => EventCode::EV_KEY(EV_KEY::BTN_SOUTH),
                Button::North => EventCode::EV_KEY(EV_KEY::BTN_NORTH),
                Button::West => EventCode::EV_KEY(EV_KEY::BTN_WEST),
                Button::East => EventCode::EV_KEY(EV_KEY::BTN_EAST),
                Button::LeftBumper => EventCode::EV_KEY(EV_KEY::BTN_TL),
                Button::RightBumper => EventCode::EV_KEY(EV_KEY::BTN_TR),
                Button::LeftStick => EventCode::EV_KEY(EV_KEY::BTN_THUMBL),
                Button::RightStick => EventCode::EV_KEY(EV_KEY::BTN_THUMBR),
                Button::Select => EventCode::EV_KEY(EV_KEY::BTN_SELECT),
                Button::Start => EventCode::EV_KEY(EV_KEY::BTN_START),
                Button::DPadDown | Button::DPadUp | Button::DPadLeft | Button::DPadRight => {
                    return self.report_dpad_input(signal);
                }
            },
            InputType::LeftStick(Axis::X) => EventCode::EV_ABS(EV_ABS::ABS_X),
            InputType::LeftStick(Axis::Y) => EventCode::EV_ABS(EV_ABS::ABS_Y),
            InputType::RightStick(Axis::X) => EventCode::EV_ABS(EV_ABS::ABS_RX),
            InputType::RightStick(Axis::Y) => EventCode::EV_ABS(EV_ABS::ABS_RY),
            InputType::Trigger(Trigger::Left) => EventCode::EV_ABS(EV_ABS::ABS_Z),
            InputType::Trigger(Trigger::Right) => EventCode::EV_ABS(EV_ABS::ABS_RZ),
        };

        let event = InputEvent::new(&Self::ZERO_TIME_VAL, &event_code, signal.value as i32);
        self.virtual_dev.write_event(&event)?;
        self.virtual_dev.write_event(&Self::SYN_REPORT_EVENT)?;
        Ok(())
    }

    fn enable_events(uninit_dev: &UninitDevice) -> Result<()> {
        let keys: [EV_KEY; 15] = [
            EV_KEY::BTN_SOUTH,
            EV_KEY::BTN_NORTH,
            EV_KEY::BTN_WEST,
            EV_KEY::BTN_EAST,
            EV_KEY::BTN_DPAD_DOWN,
            EV_KEY::BTN_DPAD_UP,
            EV_KEY::BTN_DPAD_LEFT,
            EV_KEY::BTN_DPAD_RIGHT,
            EV_KEY::BTN_SELECT,
            EV_KEY::BTN_START,
            EV_KEY::BTN_MODE,
            EV_KEY::BTN_TL,
            EV_KEY::BTN_TR,
            EV_KEY::BTN_THUMBL,
            EV_KEY::BTN_THUMBR,
        ];
        let joysticks: [EV_ABS; 4] = [EV_ABS::ABS_X, EV_ABS::ABS_Y, EV_ABS::ABS_RX, EV_ABS::ABS_RY];
        let triggers: [EV_ABS; 2] = [EV_ABS::ABS_Z, EV_ABS::ABS_RZ];
        let dpad_axes: [EV_ABS; 2] = [EV_ABS::ABS_HAT0X, EV_ABS::ABS_HAT0Y];

        let stick_abs_info = AbsInfo {
            value: 0,
            minimum: i16::MIN as i32,
            maximum: i16::MAX as i32,
            fuzz: 0,
            flat: 0,
            resolution: 0,
        };
        let trigger_abs_info = AbsInfo {
            value: 0,
            minimum: u8::MIN as i32,
            maximum: u8::MAX as i32,
            fuzz: 0,
            flat: 0,
            resolution: 0,
        };
        let dpad_abs_info = AbsInfo {
            value: 0,
            minimum: -1,
            maximum: 1,
            fuzz: 0,
            flat: 0,
            resolution: 0,
        };

        for ev_key in keys {
            uninit_dev.enable(EventCode::EV_KEY(ev_key))?;
        }
        for axis in joysticks {
            uninit_dev.enable_event_code(
                &EventCode::EV_ABS(axis),
                Some(EnableCodeData::AbsInfo(stick_abs_info)),
            )?;
        }
        for trigger in triggers {
            uninit_dev.enable_event_code(
                &EventCode::EV_ABS(trigger),
                Some(EnableCodeData::AbsInfo(trigger_abs_info)),
            )?;
        }
        for dpad in dpad_axes {
            uninit_dev.enable_event_code(
                &EventCode::EV_ABS(dpad),
                Some(EnableCodeData::AbsInfo(dpad_abs_info)),
            )?;
        }

        uninit_dev.enable(EventCode::EV_FF(EV_FF::FF_RUMBLE))?;
        uninit_dev.enable(EventCode::EV_SYN(EV_SYN::SYN_REPORT))?;
        Ok(())
    }

    fn report_dpad_input(&mut self, signal: InputSignal) -> Result<()> {
        let (abs_event_code, btn_event_code, value) = match signal.itype {
            InputType::Button(Button::DPadUp) => {
                self.dpad_state.up = signal.value == 1;
                (
                    EventCode::EV_ABS(EV_ABS::ABS_HAT0Y),
                    EventCode::EV_KEY(EV_KEY::BTN_DPAD_UP),
                    Self::compute_dpad_axis(self.dpad_state.up, self.dpad_state.down),
                )
            }
            InputType::Button(Button::DPadDown) => {
                self.dpad_state.down = signal.value == 1;
                (
                    EventCode::EV_ABS(EV_ABS::ABS_HAT0Y),
                    EventCode::EV_KEY(EV_KEY::BTN_DPAD_DOWN),
                    Self::compute_dpad_axis(self.dpad_state.up, self.dpad_state.down),
                )
            }
            InputType::Button(Button::DPadLeft) => {
                self.dpad_state.left = signal.value == 1;
                (
                    EventCode::EV_ABS(EV_ABS::ABS_HAT0X),
                    EventCode::EV_KEY(EV_KEY::BTN_DPAD_LEFT),
                    Self::compute_dpad_axis(self.dpad_state.left, self.dpad_state.right),
                )
            }
            InputType::Button(Button::DPadRight) => {
                self.dpad_state.right = signal.value == 1;
                (
                    EventCode::EV_ABS(EV_ABS::ABS_HAT0X),
                    EventCode::EV_KEY(EV_KEY::BTN_DPAD_RIGHT),
                    Self::compute_dpad_axis(self.dpad_state.left, self.dpad_state.right),
                )
            }
            _ => unreachable!(),
        };

        let abs_event = InputEvent::new(&Self::ZERO_TIME_VAL, &abs_event_code, value as i32);
        let btn_event = InputEvent::new(&Self::ZERO_TIME_VAL, &btn_event_code, signal.value as i32);
        self.virtual_dev.write_event(&abs_event)?;
        self.virtual_dev.write_event(&btn_event)?;
        self.virtual_dev.write_event(&Self::SYN_REPORT_EVENT)?;
        Ok(())
    }

    fn compute_dpad_axis(negative_pressed: bool, positive_pressed: bool) -> i8 {
        match (negative_pressed, positive_pressed) {
            (false, true) => 1,
            (true, false) => -1,
            _ => 0,
        }
    }
}
