extern crate cgi;
extern crate json;
extern crate url;

use std::{collections::HashMap, fs::File, io::{self, BufRead}, path::{Path, PathBuf}};

fn get_probability_of_string_for_year(string_to_look_for: &str, year: u32) -> (u32, u32) {
    // make sure to find the next comma
    let string_to_look_for = format!("{},", string_to_look_for);
    let filename = format!("statswithballsstrikescumulative.{}", year);
    let path : PathBuf = ["..", "statsyears", &filename].iter().collect();
    let lines = read_lines(path);
    if let Err(_) = lines {
        // This function might be called with the first year - 1, so we need to handle
        // that case and let's just be lazy and assume nothing ever goes wrong
        return (0, 0);
    }
    for line in lines.unwrap() {
        if let Ok(line) = line {
            if line.starts_with(&string_to_look_for) {
                let rest_of_line = &line[string_to_look_for.len()..];
                let parts = rest_of_line.split(',').collect::<Vec<_>>();
                if parts.len() >= 2 {
                    let total_games = parts[0].parse::<u32>();
                    let wins = parts[1].parse::<u32>();
                    if total_games.is_ok() && wins.is_ok() {
                        return (wins.unwrap(), total_games.unwrap());
                    }
                }
            }
        }
    }
    return (0, 0)
}

fn get_probability_of_string(string_to_look_for: &str, start_year: u32, end_year: u32, all_future_innings: bool) -> (u32, u32) {
    // These are cumulative files, so from start-end inclusive is
    // (end cumulative) - ((start - 1) cumulative)
    if all_future_innings {
        // TODO - This is overcounting because the same game can be in the same situation
        // multiple innings in a row.
        const MAX_INNING: u8 = 25;
        let start_inning = get_inning_from_query_string(string_to_look_for);
        let mut start_wins = 0;
        let mut start_total = 0;
        let mut end_wins = 0;
        let mut end_total = 0;
        for inning in start_inning..(MAX_INNING+1) {
            let state_string = set_inning_in_query_string(string_to_look_for, inning);
            let (temp_start_wins, temp_start_total) = get_probability_of_string_for_year(&state_string, start_year - 1);
            let (temp_end_wins, temp_end_total) = get_probability_of_string_for_year(&state_string, end_year);
            start_wins += temp_start_wins;
            start_total += temp_start_total;
            end_wins += temp_end_wins;
            end_total += temp_end_total;
        }
        let wins = end_wins - start_wins;
        let total = end_total - start_total;
        return (wins, total);
    } else {
        let (start_wins, start_total) = get_probability_of_string_for_year(string_to_look_for, start_year - 1);
        let (end_wins, end_total) = get_probability_of_string_for_year(string_to_look_for, end_year);
        let wins = end_wins - start_wins;
        let total = end_total - start_total;
        return (wins, total)
    }
}

// f64 is more precision than we need, but it keeps the strings
// looking nice
fn get_leverage_of_string(string_to_look_for: &str) -> f64 {
    // make sure to find the next comma
    let string_to_look_for = format!("{},", string_to_look_for);
    let path : PathBuf = ["..", "statsyears", "leverage"].iter().collect();
    let lines = read_lines(path);
    if let Err(_) = lines {
        // Haven't been error handling up to this point, why start now?
        return 0 as f64;
    }
    for line in lines.unwrap() {
        if let Ok(line) = line {
            if line.starts_with(&string_to_look_for) {
                let rest_of_line = &line[string_to_look_for.len()..];
                if let Ok(f) = rest_of_line.parse() {
                    return f;
                }
            }
        }
    }
    return 0 as f64;
}

fn get_inning_from_query_string(state_string: &str) -> u8 {
    state_string.split(',').collect::<Vec<_>>()[1].parse::<u8>().unwrap()
}

fn set_inning_in_query_string(state_string: &str, inning: u8) -> String {
    let mut parts = state_string.split(',').collect::<Vec<_>>();
    let inning_string = inning.to_string();
    parts[1] = &inning_string;
    parts.join(",")
}

fn process_query_string(query: &str) -> Result<json::JsonValue, String> {
    let query_parts: HashMap<String, String> = url::form_urlencoded::parse(query.as_bytes()).into_owned().collect();
    let state_string = query_parts.get("stateString").ok_or(String::from("Internal error - no stateString specified!"))?;
    let balls_strikes_state = query_parts.get("ballsStrikesState").ok_or(String::from("Internal error - no ballsStrikesState specified!"))?;
    let start_year = query_parts.get("startYear").ok_or(String::from("Internal error - no startYear specified!"))?;
    let start_year: u32 = start_year.parse().map_err(|s| format!("{}", s))?;
    let end_year = query_parts.get("endYear").ok_or(String::from("Internal error - no endYear specified!"))?;
    let end_year: u32 = end_year.parse().map_err(|s| format!("{}", s))?;
    let zero = "0".to_string();
    let all_future_innings = query_parts.get("allFutureInnings").or(Some(&zero));
    let all_future_innings: bool = all_future_innings.unwrap().parse::<u8>().unwrap() != 0;
    if all_future_innings && get_inning_from_query_string(state_string) < 9 {
        return Err("can only do all_future_innings with 9th inning or later!".to_string());
    }

    let string_to_look_for = format!("{},{}", state_string, balls_strikes_state);
    let (wins, total) = get_probability_of_string(&string_to_look_for, start_year, end_year, all_future_innings);
    // Leverage doesn't include balls and strikes
    let leverage = get_leverage_of_string(state_string);

    let result = json::object! {"wins": wins, "total": total, "leverage": leverage};
    Ok(result)
}

fn error(s: &str) -> cgi::Response {
    cgi::binary_response(200, "application/json", (json::object!{"error": s.clone()}).dump().as_bytes().to_vec())
}

fn success(s: json::JsonValue) -> cgi::Response {
    cgi::binary_response(200, "application/json", s.dump().as_bytes().to_vec())
}

// https://doc.rust-lang.org/stable/rust-by-example/std_misc/file/read_lines.html
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn process_request(request: &cgi::Request) -> Result<json::JsonValue, String> {
    let query = request.uri().query().ok_or(String::from("Internal error - no query string?"))?;
    return process_query_string(query);
}


cgi::cgi_main! { |request: cgi::Request| {
    let result = process_request(&request);
    match result {
        Ok(val) => success(val),
        Err(err) => error(&err)
    }
} }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_year() {
        let result = process_query_string("stateString=\"V\",1,0,1,0&ballsStrikesState=0,0&startYear=1957&endYear=1957&rand=0.7276145813300261").unwrap();
        assert_eq!("582", result["wins"].to_string());
        assert_eq!("1228", result["total"].to_string());
        assert_eq!("0.86", result["leverage"].to_string());
    }

    #[test]
    fn test_many_years() {
        // bottom of the 6th, 1 out, runner on 2nd, home team behind by 1 run
        let result = process_query_string("stateString=\"H\",6,1,3,-1&ballsStrikesState=0,1&startYear=1957&endYear=2019&rand=0.9792518693455747").unwrap();
        assert_eq!("251", result["wins"].to_string());
        assert_eq!("530", result["total"].to_string());
        assert_eq!("2.62", result["leverage"].to_string());
    }

    #[test]
    #[should_panic]
    fn test_too_early_to_get_all_future_innings() {
        process_query_string("stateString=\"H\",6,1,3,-1&ballsStrikesState=0,1&startYear=1957&endYear=2019&allFutureInnings=1&rand=0.9792518693455747").unwrap();
    }

    #[test]
    fn test_all_future_innings() {
        // top of the 24th, should count 25th inning too
        let result = process_query_string("stateString=\"V\",24,0,1,0&ballsStrikesState=0,0&startYear=1957&endYear=2020&allFutureInnings=1&rand=0.9792518693455747").unwrap();
        assert_eq!("2", result["wins"].to_string());
        assert_eq!("5", result["total"].to_string());
    }
}
