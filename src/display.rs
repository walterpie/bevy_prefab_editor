use bevy::prelude::*;

pub trait ToDisplay: Sized {
    fn display(&self) -> DisplayWrapper<Self>;
}

impl<T: Clone + Sized> ToDisplay for T {
    fn display(&self) -> DisplayWrapper<Self> {
        DisplayWrapper(self.clone())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayWrapper<T>(T);

impl DisplayWrapper<KeyCode> {
    pub fn as_str(&self) -> &str {
        use KeyCode::*;
        match self.0 {
            Key1 => "1",
            Key2 => "2",
            Key3 => "3",
            Key4 => "4",
            Key5 => "5",
            Key6 => "6",
            Key7 => "7",
            Key8 => "8",
            Key9 => "9",
            Key0 => "0",
            A => "a",
            B => "b",
            C => "c",
            D => "d",
            E => "e",
            F => "f",
            G => "g",
            H => "h",
            I => "i",
            J => "j",
            K => "k",
            L => "l",
            M => "m",
            N => "n",
            O => "o",
            P => "p",
            Q => "q",
            R => "r",
            S => "s",
            T => "t",
            U => "u",
            V => "v",
            W => "w",
            X => "x",
            Y => "y",
            Z => "z",
            _ => "",
        }
    }
}
