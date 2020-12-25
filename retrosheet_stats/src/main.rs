#[macro_use] extern crate lazy_static;
extern crate regex;
use std::{collections::HashSet, convert::TryInto};
use anyhow::{Error, Result};

//TODO - remove this
#[allow(unused_imports)]
//TODO - remove this
#[allow(dead_code)]

use data::{RunnerDests, RunnerFinalPosition, RunnerInitialPosition};
use regex::Regex;

mod data {
    //TODO - remove this
    #![allow(dead_code)]
    use std::{collections::HashMap, convert::TryFrom};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum RunnerInitialPosition {
        Batter,
        FirstBase,
        SecondBase,
        ThirdBase
    }

    impl TryFrom<char> for RunnerInitialPosition {
        type Error = anyhow::Error;

        fn try_from(value: char) -> Result<Self, Self::Error> {
            match value {
                'B' => Ok(RunnerInitialPosition::Batter),
                '1' => Ok(RunnerInitialPosition::FirstBase),
                '2' => Ok(RunnerInitialPosition::SecondBase),
                '3' => Ok(RunnerInitialPosition::ThirdBase),
                _ => Err(anyhow::Error::msg("Unrecognized char for RunnerInitialPosition"))
            }
        }
    }

    impl RunnerInitialPosition {
        pub fn base_number(self: &Self) -> usize {
            match *self {
                RunnerInitialPosition::Batter => {
                    0
                },
                RunnerInitialPosition::FirstBase => {
                    1
                },
                RunnerInitialPosition::SecondBase => {
                    2
                },
                RunnerInitialPosition::ThirdBase => {
                    3
                },
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum RunnerFinalPosition {
        FirstBase,
        SecondBase,
        ThirdBase,
        HomePlate, // this means the runner scored
        StillAtBat, // only valid for Batter
        Undetermined,
        Out
    }

    impl TryFrom<char> for RunnerFinalPosition {
        type Error = anyhow::Error;

        fn try_from(value: char) -> Result<Self, Self::Error> {
            match value {
                '1' => Ok(RunnerFinalPosition::FirstBase),
                '2' => Ok(RunnerFinalPosition::SecondBase),
                '3' => Ok(RunnerFinalPosition::ThirdBase),
                'H' => Ok(RunnerFinalPosition::HomePlate),
                _ => Err(anyhow::Error::msg("Unrecognized char for RunnerFinalPosition"))
            }
        }
    }



    impl RunnerFinalPosition {
        pub fn runner_index(self: &Self) -> usize {
            match *self {
                RunnerFinalPosition::FirstBase => {
                    0
                },
                RunnerFinalPosition::SecondBase => {
                    1
                },
                RunnerFinalPosition::ThirdBase => {
                    2
                },
                _ => {
                    //TODO
                    panic!("runner_index() called on {:?}", self);
                }
            }
        }
        pub fn base_number(self: &Self) -> usize {
            match *self {
                RunnerFinalPosition::FirstBase => {
                    1
                },
                RunnerFinalPosition::SecondBase => {
                    2
                },
                RunnerFinalPosition::ThirdBase => {
                    3
                },
                RunnerFinalPosition::HomePlate => {
                    4
                },
                _ => {
                    //TODO
                    panic!("base_number() called on {:?}", self);
                }
            }

        }
    }

    pub struct RunnerDests {
        // Putting this struct in a module so its implementation is hidden.
        // TODO - Probably want to move this to a simple array with 4 entries
        dests: HashMap<RunnerInitialPosition, RunnerFinalPosition>
    }

    impl RunnerDests {
        pub fn new_from_runners(runners: &[bool;3]) -> RunnerDests {
            let mut dests = HashMap::new();
            if runners[0] {
                // TODO - this is a change from python, used to be StillAtBat (-1)
                dests.insert(RunnerInitialPosition::FirstBase, RunnerFinalPosition::Undetermined);
            }
            if runners[1] {
                // TODO - this is a change from python, used to be StillAtBat (-1)
                dests.insert(RunnerInitialPosition::SecondBase, RunnerFinalPosition::Undetermined);
            }
            if runners[2] {
                // TODO - this is a change from python, used to be StillAtBat (-1)
                dests.insert(RunnerInitialPosition::ThirdBase, RunnerFinalPosition::Undetermined);
            }
            RunnerDests { dests }
        }

        pub fn batter_to_first(self: &mut Self) {
            self.dests.insert(RunnerInitialPosition::Batter, RunnerFinalPosition::FirstBase);
            if let Some(entry) = self.dests.get_mut(&RunnerInitialPosition::FirstBase) {
                *entry = RunnerFinalPosition::SecondBase;
                if let Some(entry) = self.dests.get_mut(&RunnerInitialPosition::SecondBase) {
                    *entry = RunnerFinalPosition::ThirdBase;
                    if let Some(entry) = self.dests.get_mut(&RunnerInitialPosition::ThirdBase) {
                        *entry = RunnerFinalPosition::HomePlate;
                    }
                }
                else {
                    if let Some(entry) = self.dests.get_mut(&RunnerInitialPosition::ThirdBase) {
                        *entry = RunnerFinalPosition::ThirdBase;
                    }
                }
            }
            else {
                if let Some(entry) = self.dests.get_mut(&RunnerInitialPosition::SecondBase) {
                    *entry = RunnerFinalPosition::SecondBase;
                }
                if let Some(entry) = self.dests.get_mut(&RunnerInitialPosition::ThirdBase) {
                    *entry = RunnerFinalPosition::ThirdBase;
                }
            }
        }

        pub fn len(self: &Self) -> usize {
            self.dests.len()
        }

        pub fn get(self: &Self, key: RunnerInitialPosition) -> Option<RunnerFinalPosition> {
            self.dests.get(&key).map(|x| *x)
        }

        pub fn keys(self: &Self) -> impl Iterator<Item=RunnerInitialPosition> + '_ {
            self.dests.keys().map(|x| *x).into_iter()
        }

        pub fn set_all<F>(self: &mut Self, func: F)
            where F: Fn(RunnerInitialPosition) -> RunnerFinalPosition {
            //self.dests.entry(key)
            for (&key, value) in self.dests.iter_mut() {
                *value = func(key);
            }
        }

        pub fn set(self: &mut Self, key: RunnerInitialPosition, value: RunnerFinalPosition) {
            self.dests.insert(key, value);
        }
    }

}

//TODO - remove this
#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
enum Verbosity {
    Quiet = 0,
    Normal = 1,
    Verbose = 2
}

impl Verbosity {
    fn is_at_least(self: &Self, compare: Verbosity) -> bool {
        return *self as u8 >= compare as u8;
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct GameSituation {
    // Whether runners are on first, second, third bases
    runners: [bool;3],
    inning: u8,
    cur_score_diff: i8,
    outs: u8, // TODO - should this be an enum?
    is_home: bool,
}

impl GameSituation {
    //TODO - remove this
    #![allow(dead_code)]

    fn new() -> GameSituation {
        GameSituation {
            cur_score_diff: 0,
            inning: 1,
            is_home: false,
            outs: 0,
            runners: [false, false, false]
        }
    }

    // Advances to the next inning if there are 3 outs
    fn next_inning_if_three_outs(self: &mut Self) {
        if self.outs >= 3 {
            if self.is_home {
                self.is_home = false;
                self.inning += 1;
            }
            else {
                self.is_home = true;
            }
            self.outs = 0;
            self.runners[0] = false;
            self.runners[1] = false;
            self.runners[2] = false;
            self.cur_score_diff = -1 * self.cur_score_diff;
        }
    }

    // Whether the home team won, or None if it's still tied
    fn is_home_winning(self: &Self) -> Option<bool> {
        if self.cur_score_diff == 0 {
            // This game must have been tied when it stopped.
            None
        }
        else {
            if self.is_home {
                Some(self.cur_score_diff > 0)
            }
            else {
                Some(self.cur_score_diff < 0)
            }
        }
    }

    fn parse_play(self: &mut GameSituation, line: &str, verbosity: Verbosity) -> Result<()> {
        // decription of the format is at http://www.retrosheet.org/eventfile.htm
        let play_line_info = PlayLineInfo::new_from_line(line);
        let mut runner_dests = RunnerDests::new_from_runners(&self.runners);
        // TODO perf - use a Vec<> or something? Or do we even need this, can we just use runner_dests?
        let beginning_runners = runner_dests.keys().collect::<HashSet<_>>();
        let mut runners_default_stay_still = false;
        //TODO - verbosity log statements
        if verbosity.is_at_least(Verbosity::Verbose) {
            println!("Game situation is {:?}", self);
            println!("{}", line);
        }

        // TODO - should return a result?
        assert_eq!(self.inning, play_line_info.inning);
        assert_eq!(self.is_home, play_line_info.is_home);

        let play_string = &play_line_info.play_str;
        // TODO perf - is this collect() necessary?
        let play_array: Vec<&str> = play_string.split('.').collect();
        assert!(play_array.len() <= 2);
        // Deal with the first part of the string.
        let batter_events = play_array[0].split(';');
        for batter_event in batter_events {
            let batter_event = batter_event.trim();
            let mut done_parsing_event = false;
            lazy_static! {
                static ref SIMPLE_HIT_RE : Regex = Regex::new(r"([SDTH])(?:\d|/)").unwrap();
                static ref SIMPLE_HIT_2_RE : Regex = Regex::new(r"([SDTH])\s*$").unwrap();
            }
            let simple_hit_match = SIMPLE_HIT_RE.captures(batter_event);
            let simple_hit_2_match = SIMPLE_HIT_RE.captures(batter_event);
            let captures = simple_hit_match.or(simple_hit_2_match);
            if let Some(inner_captures) = captures {
                let type_of_hit = inner_captures.get(1).unwrap().as_str();
                match type_of_hit {
                    "S" => {
                        runner_dests.set(RunnerInitialPosition::Batter, RunnerFinalPosition::FirstBase);
                    },
                    "D" => {
                        runner_dests.set(RunnerInitialPosition::Batter, RunnerFinalPosition::SecondBase);
                    },
                    "T" => {
                        runner_dests.set(RunnerInitialPosition::Batter, RunnerFinalPosition::ThirdBase);
                    },
                    "H" => {
                        runner_dests.set_all(|_| RunnerFinalPosition::HomePlate);
                    },
                    _ => panic!("Unexpected type_of_hit {}", type_of_hit)
                }
                // Sometimes these aren't specified - assume runners don't move
                runners_default_stay_still = true;
                done_parsing_event = true;
            }
            if !done_parsing_event {
                if batter_event.starts_with("HR") {
                    runner_dests.set_all(|_| RunnerFinalPosition::HomePlate);
                }
                done_parsing_event = true;
            }
            // TODO - much more
        }

        // TODO - Now parse runner stuff
        if play_array.len() > 1 {
            let runner_array = play_array[1].split(';').into_iter().map(|x| x.trim());
            for runner_item in runner_array {
                let runner_chars = runner_item.chars().collect::<Vec<_>>();
                if runner_chars.len() != 3 {
                    assert_eq!('(', runner_chars[3]);
                }
                let initial_runner: RunnerInitialPosition = runner_chars[0].try_into()?;
                let final_runner: RunnerFinalPosition = runner_chars[2].try_into()?;
                match runner_chars[1] {
                    '-' => {
                        // This looks weird, but sometimes a runner can go to the
                        // same base (a little redundant, but OK)
                        if initial_runner.base_number() > final_runner.base_number() {
                            return Err(anyhow::Error::msg(format!("Runner went backwards from {:?} to {:?} for play {}", initial_runner, final_runner, play_string)));
                        }
                        runner_dests.set(initial_runner, final_runner);
                    },
                    'X' => {
                        //TODO
                    },
                    //TODO better message
                    _ => return Err(anyhow::Error::msg(format!("Invalid character {} in runner specification for play {}", runner_chars[1], play_string)))
                };
            }
        }

        // TODO even more stuff

        // Deal with runner_dests
        // TODO - move this into a method
        self.runners = [false, false, false];
        for key in runner_dests.keys() {
            let dest = runner_dests.get(key).unwrap();
            match dest {
                RunnerFinalPosition::Out => {
                    self.outs += 1;
                },
                RunnerFinalPosition::HomePlate => {
                    self.cur_score_diff += 1;
                },
                RunnerFinalPosition::Undetermined | RunnerFinalPosition::StillAtBat => {
                    // Either we're the batter, and nothing happens, or
                    // we don't know what happens, and it doesn't matter because there
                    // are three outs.
                },
                RunnerFinalPosition::FirstBase | RunnerFinalPosition::SecondBase | RunnerFinalPosition::ThirdBase => {
                    if *self.runners.get(dest.runner_index()).unwrap() {
                        if verbosity.is_at_least(Verbosity::Normal) {
                            println!("ERROR - already a runner at base {}!", dest.runner_index());
                        }
                        return Err(Error::msg("ERROR - duplicate runner"));
                    }
                    *(self.runners.get_mut(dest.runner_index()).unwrap()) = true;
                }
            }
        }
        self.next_inning_if_three_outs();
        Ok(())
    }

}
#[derive(Clone, Debug, PartialEq, Eq)]
struct PlayLineInfo<'a> {
    inning: u8,
    is_home: bool,
    player_id: &'a str,
    count_when_play_happened: &'a str,
    pitches_str: &'a str,
    play_str: String
}

impl PlayLineInfo<'_> {
    fn new_from_line<'a>(line: &'a str) -> PlayLineInfo<'a> {
        lazy_static! {
            // TODO perf - opt out of unicode?
            static ref PLAY_RE : Regex = Regex::new(r"^play,\s?(\d+),\s?([01]),(.*?),(.*?),(.*?),(.*)$").unwrap();
        }
        let play_match = PLAY_RE.captures(line).unwrap();
        // remove characters we don't care about
        let play_str = play_match.get(6).unwrap().as_str().chars()
            .filter(|&x| x != '!' && x != '#' && x != '?').collect();
        return PlayLineInfo {
            inning: play_match.get(1).unwrap().as_str().parse::<u8>().unwrap(),
            is_home: play_match.get(2).unwrap().as_str() == "1",
            player_id: play_match.get(3).unwrap().as_str(),
            count_when_play_happened: play_match.get(4).unwrap().as_str(),
            pitches_str: play_match.get(5).unwrap().as_str(),
            play_str: play_str
        }
    }
}


// TODO - parallel

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]
    use std::collections::HashMap;
    use data::*;
    use super::*;

    #[test]
    fn test_next_inning_if_three_outs__zero_outs() {
        let orig_inning = GameSituation {
            cur_score_diff: 2,
            inning: 1,
            is_home: false,
            outs: 0,
            runners: [false, true, false]
        };
        let mut new_inning = orig_inning.clone();
        new_inning.next_inning_if_three_outs();
        assert_eq!(orig_inning, new_inning);
    }

    #[test]
    fn test_next_inning_if_three_outs__one_out() {
        let orig_inning = GameSituation {
            cur_score_diff: 2,
            inning: 1,
            is_home: false,
            outs: 1,
            runners: [false, true, false]
        };
        let mut new_inning = orig_inning.clone();
        new_inning.next_inning_if_three_outs();
        assert_eq!(orig_inning, new_inning);
    }

    #[test]
    fn test_next_inning_if_three_outs__two_outs() {
        let orig_inning = GameSituation {
            cur_score_diff: 2,
            inning: 1,
            is_home: false,
            outs: 2,
            runners: [false, true, false]
        };
        let mut new_inning = orig_inning.clone();
        new_inning.next_inning_if_three_outs();
        assert_eq!(orig_inning, new_inning);
    }

    #[test]
    fn test_next_inning_if_three_outs__three_outs_home() {
        let mut orig_inning = GameSituation {
            cur_score_diff: 2,
            inning: 1,
            is_home: true,
            outs: 3,
            runners: [false, true, false]
        };
        orig_inning.next_inning_if_three_outs();
        assert_eq!(GameSituation {
            cur_score_diff: -2,
            inning: 2,
            is_home: false,
            outs: 0,
            runners: [false, false, false]
        }, orig_inning);
    }

    #[test]
    fn test_next_inning_if_three_outs__three_outs_visitor() {
        let mut orig_inning = GameSituation {
            cur_score_diff: 2,
            inning: 1,
            is_home: false,
            outs: 3,
            runners: [false, true, false]
        };
        orig_inning.next_inning_if_three_outs();
        assert_eq!(GameSituation {
            cur_score_diff: -2,
            inning: 1,
            is_home: true,
            outs: 0,
            runners: [false, false, false]
        }, orig_inning);
    }

    #[test]
    fn test_is_home_winning__home_inning_tied() {
        let mut game = GameSituation::new();
        game.is_home = true;
        game.cur_score_diff = 0;
        assert_eq!(None, game.is_home_winning());
    }

    #[test]
    fn test_is_home_winning__visitor_inning_tied() {
        let mut game = GameSituation::new();
        game.is_home = false;
        game.cur_score_diff = 0;
        assert_eq!(None, game.is_home_winning());
    }

    #[test]
    fn test_is_home_winning__home_inning_home_ahead() {
        let mut game = GameSituation::new();
        game.is_home = true;
        game.cur_score_diff = 2;
        assert_eq!(Some(true), game.is_home_winning());
    }

    #[test]
    fn test_is_home_winning__home_inning_visitor_ahead() {
        let mut game = GameSituation::new();
        game.is_home = true;
        game.cur_score_diff = -2;
        assert_eq!(Some(false), game.is_home_winning());
    }

    #[test]
    fn test_is_home_winning__visitor_inning_home_ahead() {
        let mut game = GameSituation::new();
        game.is_home = false;
        game.cur_score_diff = -2;
        assert_eq!(Some(true), game.is_home_winning());
    }

    #[test]
    fn test_is_home_winning__visitor_inning_visitor_ahead() {
        let mut game = GameSituation::new();
        game.is_home = false;
        game.cur_score_diff = 2;
        assert_eq!(Some(false), game.is_home_winning());
    }

    #[test]
    fn test_batter_to_first() {
        // Yikes, this type is something
        let data: Vec<([bool;3], Box<dyn Iterator<Item=&(RunnerInitialPosition, RunnerFinalPosition)>>)> = vec![
            ([false, false, false],
             Box::new([(RunnerInitialPosition::Batter, RunnerFinalPosition::FirstBase)].iter())),
            ([true, false, false],
             Box::new([(RunnerInitialPosition::Batter, RunnerFinalPosition::FirstBase),
                       (RunnerInitialPosition::FirstBase, RunnerFinalPosition::SecondBase)].iter())),
            ([false, true, false],
             Box::new([(RunnerInitialPosition::Batter, RunnerFinalPosition::FirstBase),
                       (RunnerInitialPosition::SecondBase, RunnerFinalPosition::SecondBase)].iter())),
            ([true, true, false],
             Box::new([(RunnerInitialPosition::Batter, RunnerFinalPosition::FirstBase),
                       (RunnerInitialPosition::FirstBase, RunnerFinalPosition::SecondBase),
                       (RunnerInitialPosition::SecondBase, RunnerFinalPosition::ThirdBase)].iter())),
            ([false, false, true],
             Box::new([(RunnerInitialPosition::Batter, RunnerFinalPosition::FirstBase),
                       (RunnerInitialPosition::ThirdBase, RunnerFinalPosition::ThirdBase)].iter())),
            ([true, false, true],
             Box::new([(RunnerInitialPosition::Batter, RunnerFinalPosition::FirstBase),
                       (RunnerInitialPosition::FirstBase, RunnerFinalPosition::SecondBase),
                       (RunnerInitialPosition::ThirdBase, RunnerFinalPosition::ThirdBase)].iter())),
            ([false, true, true],
             Box::new([(RunnerInitialPosition::Batter, RunnerFinalPosition::FirstBase),
                       (RunnerInitialPosition::SecondBase, RunnerFinalPosition::SecondBase),
                       (RunnerInitialPosition::ThirdBase, RunnerFinalPosition::ThirdBase)].iter())),
            ([true, true, true],
             Box::new([(RunnerInitialPosition::Batter, RunnerFinalPosition::FirstBase),
                       (RunnerInitialPosition::FirstBase, RunnerFinalPosition::SecondBase),
                       (RunnerInitialPosition::SecondBase, RunnerFinalPosition::ThirdBase),
                       (RunnerInitialPosition::ThirdBase, RunnerFinalPosition::HomePlate)].iter()))
        ];
        for (runners, expectedIter) in data {
            let mut dests = RunnerDests::new_from_runners(&runners);
            dests.batter_to_first();
            let expected = expectedIter.map(|x| *x).collect::<HashMap<_,_>>();
            assert_eq!(expected.len(), dests.len(), "{:?}", runners);
            for (key, expectedValue) in expected {
                assert_eq!(Some(expectedValue), dests.get(key), "{:?} {:?}", runners, key);
            }
        }
    }

    #[test]
    fn test_parse_play_line_info() {
        let play_line_info_str = "play,4,1,corrc001,22,BSBFFX,HR/78/F";
        let play_line_info = PlayLineInfo::new_from_line(play_line_info_str);
        let expected = PlayLineInfo {
            inning: 4,
            is_home: true,
            player_id: "corrc001",
            count_when_play_happened: "22",
            pitches_str: "BSBFFX",
            play_str: "HR/78/F".to_owned()
        };
        assert_eq!(expected, play_line_info);
    }

    #[test]
    fn test_verbosity_is_at_least() {
        assert_eq!(true, Verbosity::Verbose.is_at_least(Verbosity::Verbose));
        assert_eq!(true, Verbosity::Verbose.is_at_least(Verbosity::Normal));
        assert_eq!(true, Verbosity::Verbose.is_at_least(Verbosity::Quiet));
        assert_eq!(false, Verbosity::Normal.is_at_least(Verbosity::Verbose));
        assert_eq!(true, Verbosity::Normal.is_at_least(Verbosity::Normal));
        assert_eq!(true, Verbosity::Normal.is_at_least(Verbosity::Quiet));
        assert_eq!(false, Verbosity::Quiet.is_at_least(Verbosity::Verbose));
        assert_eq!(false, Verbosity::Quiet.is_at_least(Verbosity::Normal));
        assert_eq!(true, Verbosity::Quiet.is_at_least(Verbosity::Quiet));
    }

    mod parse_play_tests {
        #![allow(non_snake_case)]
        use super::*;

        fn setup(outs: u8, is_home: bool, play_string: &str) -> (GameSituation, String) {
            let situation = GameSituation {
                // Whether runners are on first, second, third bases
                runners: [false, false, false],
                inning: 1,
                cur_score_diff: 0,
                outs: outs,
                is_home: is_home,
            };

            (situation, format!("play,1,{},,,,{}", if situation.is_home { 1 } else { 0 }, play_string))
        }

        #[test]
        #[ignore]
        fn test_simpleout() -> Result<()> {
            let (mut situation, play_line) = setup(0, false, "8");
            let mut expected_situation = situation.clone();
            expected_situation.outs = 1;
            situation.parse_play(&play_line, Verbosity::Normal)?;
            assert_eq!(expected_situation, situation);
            Ok(())
        }

        #[test]
        fn test_single() -> Result<()> {
            let (mut situation, play_line) = setup(0, false, "S7");
            let mut expected_situation = situation.clone();
            expected_situation.runners[0] = true;
            situation.parse_play(&play_line, Verbosity::Normal)?;
            assert_eq!(expected_situation, situation);
            Ok(())
        }

        #[test]
        fn test_double() -> Result<()> {
            let (mut situation, play_line) = setup(0, false, "D7/G5.3-H;2-H;1-H");
            situation.runners = [true, true, true];
            let mut expected_situation = situation.clone();
            expected_situation.runners = [false, true, false];
            expected_situation.cur_score_diff = 3;
            situation.parse_play(&play_line, Verbosity::Normal)?;
            assert_eq!(expected_situation, situation);
            Ok(())
        }
    }
}