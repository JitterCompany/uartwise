
use embedded_hal::digital::v2::InputPin;
use core::convert::Infallible;

pub struct Encoder<CHA: InputPin, CHB: InputPin> {
    channel_a: CHA,
    channel_b: CHB,
    position: i32,
}

pub enum Channel {
    A,
    B
}

impl<CHA, CHB> Encoder<CHA, CHB>
where
    CHA: InputPin<Error = Infallible>,
    CHB: InputPin<Error = Infallible>,
{
    pub fn new(ch_a: CHA, ch_b: CHB) -> Self {
        Self {
            channel_a: ch_a,
            channel_b: ch_b,
            position: 0,
        }
    }

    pub fn update(&mut self, ch: Channel) -> (i32, i32) {
        let a: bool = self.channel_a.is_high().unwrap();
        let b: bool = self.channel_b.is_high().unwrap();
        let mut step = 1;

        match ch {
            Channel::A => {
                if a == b {
                    step = -1;
                }
            },
            Channel::B => {
                if a != b {
                    step = -1;
                }
            }
        }

        self.position += step;

        (self.position, step)
    }
}