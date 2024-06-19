use regex::Regex;

#[derive(Default)]
enum SearchState {
    #[default]
    General,
    DnsSuffixSearchList,
    DnsServers,
}

pub struct Search<'a> {
    suffixes: Vec<&'a str>,
    servers: Vec<&'a str>,

    state: SearchState,

    servers_header: Regex,
    servers_extra: Regex,
    suffixes_header: Regex,
    suffixes_extra: Regex,
}

const SERVERS_HEADER_REGEX: &str =
    r"DNS Servers(.+)[^0-9](((25[0-5]|(2[0-4]|1\d|[1-9]|)\d)\.?\b){4})";
const SERVERS_EXTRA_REGEX: &str = r"^[^0-9]+(((25[0-5]|(2[0-4]|1\d|[1-9]|)\d)\.?\b){4})";
const SUFFIXES_HEADER_REGEX: &str = r"DNS Suffix Search List(.)+: ((([a-z])+\.?)+)";
const SUFFIXES_EXTRA_REGEX: &str = r"^\s+((([a-z])+\.?)+)";

impl<'a> Default for Search<'a> {
    fn default() -> Self {
        Self {
            suffixes: Default::default(),
            servers: Default::default(),
            state: Default::default(),

            // These .expects are acceptable because they are hardcoded.
            servers_header: Regex::new(SERVERS_HEADER_REGEX).unwrap(),
            servers_extra: Regex::new(SERVERS_EXTRA_REGEX).unwrap(),
            suffixes_header: Regex::new(SUFFIXES_HEADER_REGEX).unwrap(),
            suffixes_extra: Regex::new(SUFFIXES_EXTRA_REGEX).unwrap(),
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

    pub(crate) fn generate_resolv_conf(
        &self,
        output: Box<dyn std::io::Write>,
    ) -> Result<(), anyhow::Error> {
        crate::resolv::generate(output, &self.suffixes, &self.servers)
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
