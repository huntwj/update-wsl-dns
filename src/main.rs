use std::process::Command;

use regex::Regex;

mod resolv;

#[derive(Default)]
enum SearchState {
    #[default]
    General,
    DnsSuffixSearchList,
    DnsServers,
}

struct Search<'a> {
    suffixes: Vec<&'a str>,
    servers: Vec<&'a str>,

    state: SearchState,

    servers_header: Regex,
    servers_extra: Regex,
    suffixes_header: Regex,
    suffixes_extra: Regex,
}

impl<'a> Default for Search<'a> {
    fn default() -> Self {
        Self {
            suffixes: Default::default(),
            servers: Default::default(),
            state: Default::default(),

            // These .expects are acceptable because they are hardcoded.
            servers_header: Regex::new(
                r"DNS Servers(.+)[^0-9](((25[0-5]|(2[0-4]|1\d|[1-9]|)\d)\.?\b){4})",
            )
            .expect("Invalid DNS Servers header regex"),
            servers_extra: Regex::new(r"^[^0-9]+(((25[0-5]|(2[0-4]|1\d|[1-9]|)\d)\.?\b){4})")
                .expect("Invalid extra DNS server regex"),
            suffixes_header: Regex::new(r"DNS Suffix Search List(.)+: ((([a-z])+\.?)+)")
                .expect("Invalid DNS Suffixes header regex"),
            suffixes_extra: Regex::new(r"^\s+((([a-z])+\.?)+)")
                .expect("Invalid extra DNS Suffix regex"),
        }
    }
}

const BAD_CAPTURE: &str = "Regex capture not in expected location";

impl<'a> Search<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    fn match_servers_header(&mut self, line: &'a str) -> Option<()> {
        self.servers_header.captures(line).map(|captures| {
            self.servers
                .push(captures.get(2).expect(BAD_CAPTURE).as_str());
            self.state = SearchState::DnsServers
        })
    }

    fn match_servers_extra(&mut self, line: &'a str) -> Option<()> {
        self.servers_extra.captures(line).map(|captures| {
            self.servers
                .push(captures.get(1).expect(BAD_CAPTURE).as_str());
        })
    }

    fn match_suffixes_header(&mut self, line: &'a str) -> Option<()> {
        self.suffixes_header.captures(line).map(|captures| {
            self.suffixes
                .push(captures.get(2).expect(BAD_CAPTURE).as_str());

            self.state = SearchState::DnsSuffixSearchList;
        })
    }

    fn match_suffixes_extra(&mut self, line: &'a str) -> Option<()> {
        self.suffixes_extra.captures(line).map(|captures| {
            self.suffixes
                .push(captures.get(1).expect(BAD_CAPTURE).as_str());
        })
    }

    pub fn process_line(&mut self, line: &'a str) {
        match self.state {
            SearchState::General => self
                .match_servers_header(line)
                .or_else(|| self.match_suffixes_header(line))
                .unwrap_or(()),

            SearchState::DnsSuffixSearchList => self
                .match_suffixes_extra(line)
                .or_else(|| self.match_servers_header(line))
                .unwrap_or_else(|| {
                    // Reset to General search mode
                    self.state = SearchState::General;
                }),
            SearchState::DnsServers => self
                .match_servers_extra(line)
                .or_else(|| self.match_suffixes_header(line))
                .unwrap_or_else(|| {
                    // Reset to General search mode
                    self.state = SearchState::General;
                }),
        };
    }

    fn generate_resolv_conf(&self, output: Box<dyn std::io::Write>) -> Result<(), anyhow::Error> {
        resolv::generate(output, &self.suffixes, &self.servers)
    }
}

impl<'a> From<&'a str> for Search<'a> {
    fn from(value: &'a str) -> Self {
        let mut search = Search::new();
        let lines = value.split('\n');
        for line in lines {
            search.process_line(line);
        }

        search
    }
}

fn main() -> anyhow::Result<()> {
    let command = "/mnt/c/Windows/System32/ipconfig.exe";
    let args = ["/all"];

    let out = Command::new(command)
        .args(args)
        .output()
        .expect("Error running ipconfig.exe command.");

    let all_output = String::from_utf8(out.stdout)
        .expect("Could not create string stdout. Perhaps it was not UTF-8 encoded?");

    let search = Search::from(all_output.as_str());

    search.generate_resolv_conf(Box::new(std::io::stdout()))?;

    Ok(())
}
