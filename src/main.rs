use std::{env, process::ExitCode};

#[inline]
fn run(endpoint: &str) -> Result<minreq::Response, minreq::Error> {
    minreq::get(endpoint).with_timeout(3).send()
}

fn main() -> ExitCode {
    let response = run(&env::args().next_back().unwrap());
    match response {
        Ok(r) => {
            if r.status_code >= 300 {
                return ExitCode::from(1);
            }
        }
        Err(_) => {
            return ExitCode::from(1);
        }
    }

    ExitCode::from(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_reach_google() {
        let res = run("https://google.com");
        assert!(res.is_ok())
    }

    #[test]
    fn cant_reach_nonsense() {
        let res = run("https://asdqeqweqweqweqwe.local/qweqweqweqwewqe");
        assert!(res.is_err())
    }
}
