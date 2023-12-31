use std::str::{FromStr};


#[derive(PartialEq, Debug, Default)]
pub struct ClockControl {
    pub white_time: Option<u64>,
    pub black_time: Option<u64>,
    pub white_inc: Option<u64>,
    pub black_inc: Option<u64>,
    pub movestogo: Option<u64>,
}

#[derive(PartialEq, Debug)]
pub enum TimeControl {
    Infinite,
    FixedDepth(u64),
    FixedNodes(u64),
    FixedTime(u64),
    Variable(ClockControl)
}



impl FromStr for TimeControl {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tokens = s.split(" ");
        let mut cc = ClockControl::default();

        fn parse_numeric<T: FromStr>(tk: &mut std::str::Split<'_, &str>) -> Result<T, &'static str> {
            tk.next().ok_or("No Value found")?.parse::<T>().or(Err("Unparseable value"))
        }

        while let Some(tk) = tokens.next() {
            match tk {
                "infinite" => return Ok(Self::Infinite),
                "depth" => return Ok(Self::FixedDepth(parse_numeric(&mut tokens)?)),
                "nodes" => return Ok(Self::FixedNodes(parse_numeric(&mut tokens)?)),
                "movetime" => return Ok(Self::FixedTime(parse_numeric(&mut tokens)?)),
                "wtime" => cc.white_time = Some(parse_numeric::<i32>(&mut tokens)?.max(0) as u64),
                "btime" => cc.black_time = Some(parse_numeric::<i32>(&mut tokens)?.max(0) as u64),
                "winc" => cc.white_inc = Some(parse_numeric(&mut tokens)?),
                "binc" => cc.black_inc = Some(parse_numeric(&mut tokens)?),
                "movestogo" => cc.movestogo = Some(parse_numeric(&mut tokens)?),
                _ => return Err("Unknown time control")
            }
        }

        if cc.black_time.is_none() || cc.white_time.is_none() {
            return Err("Missing timecontrol values");
        }

        Ok(TimeControl::Variable(cc))
    }
}