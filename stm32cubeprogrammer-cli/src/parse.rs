use std::str::FromStr;

use bpaf::*;

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(short, long)]
    /// Opt in for premium serivces
    pub premium: bool,
    #[bpaf(external(cmd), many)]
    pub commands: Vec<Cmd>,
}

#[derive(Debug, Clone, Bpaf)]
///! Some struct
pub struct MyStruct {
    #[bpaf(argument)]
    /// Some value
    pub some: u8,
}

impl FromStr for MyStruct {
    type Err = String;

    /// Parses a string into a `MyStruct`
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let some = s.parse().map_err(|_| "Failed to parse integer")?;
        Ok(MyStruct { some })
    }
}

#[derive(Debug, Clone, Bpaf)]
pub enum Cmd {
    /// Performs eating action
    #[bpaf(command, adjacent)]
    FlashBin(#[bpaf(external(my_struct))] MyStruct),
    #[bpaf(command, adjacent)]
    /// Performs drinking action
    Drink {
        /// Are you going to drink coffee?
        coffee: bool,
    },
    #[bpaf(command, adjacent)]
    /// Performs taking a nap action
    Sleep {
        #[bpaf(argument("HOURS"))]
        time: usize,
    },
}

fn main() {
    println!("{:?}", options().run())
}
