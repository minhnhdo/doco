use std::io::{self, Read};
use std::fs;
use std::fmt;
use std::collections::HashMap;
use regex::Regex;

pub struct Invariants {}

// maps a name (object or method) to a list of inferred pre- and post-conditions
type InvariantList = HashMap<String, Inferences>;

const DAIKON_OBJ: &str = "OBJECT";
const DAIKON_ENTER: &str = "ENTER";
const DAIKON_EXIT: &str = "EXIT";

const DAIKON_NULL: &str = "null";
const DAIKON_EQ: &str = "==";
const DAIKON_NOTEQ: &str = "!=";

type Expression = String;

struct Inferences {
    pre: Vec<Invariant>,  // list of pre-conditions
    post: Vec<Invariant>, // list of post-conditions
}

enum Invariant {
    Null {
        exp: Expression,
    }, // x is NULL
    NotNull {
        exp: Expression,
    }, // x is not NULL
    Comparison {
        lhs: Expression,
        operator: String,
        rhs: Expression,
    }, // x >= y
    Original {
        source: Expression,
        target: Expression,
    }, // x == orig(y)
}
enum InfType {
    PreCondition,
    PostCondition,
}

impl fmt::Display for Invariant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Invariant::Null { ref exp } => write!(f, "{} is NULL", exp),
            Invariant::NotNull { ref exp } => write!(f, "{} is not NULL", exp),
            Invariant::Comparison {
                ref lhs,
                ref operator,
                ref rhs,
            } => write!(f, "{} {} {}", lhs, operator, rhs),
            Invariant::Original {
                ref source,
                ref target,
            } => {
                if source == target {
                    return write!(f, "{} is unchanged", source);
                }

                write!(f, "{} == original value of {}", source, target)
            }
        }
    }
}

impl Inferences {
    fn new() -> Inferences {
        Inferences {
            pre: Vec::new(),
            post: Vec::new(),
        }
    }

    fn add_precond(&mut self, inv: Invariant) {
        self.pre.push(inv);
    }

    fn add_postcond(&mut self, inv: Invariant) {
        self.post.push(inv);
    }
}

impl Invariants {
    fn add_entity(&mut self, name: &str) {
        let infs = Inferences::new();
        self.invariants.insert(name.to_string(), &mut infs);
    }

    fn add_inv(&self, name: &str, inv: Invariant, inftype: InfType) {
        if let Some(inf) = self.invariants.get(name) {
            match inftype {
                InfType::PreCondition => inf.add_precond(inv),
                InfType::PostCondition => inf.add_postcond(inv),
            };
        }
    }

    fn parse(daikon_inv: &str) -> InvariantList {
        let mut ret = HashMap::new();
        let mut dstarted = false; // daikon invariants started
        let sep = Regex::new(r"^=+$").unwrap();
        let entity_def = Regex::new(r"^(\S+):::([\S]+)$").unwrap();
        let mut inftype = InfType::PreCondition;
        let mut curr_entity = "";

        for (i, line) in daikon_inv.split("\n").enumerate() {
            // skip Daikon header
            if !dstarted {
                if sep.is_match(line) {
                    dstarted = true;
                }

                continue;
            }

            // separator (====)
            if sep.is_match(line) {
                continue;
            }

            // Object/Method start/end
            if entity_def.is_match(line) {
                for cap in entity_def.captures_iter(line) {
                    match &cap[2] {
                        DAIKON_OBJ => ret.add_entity(&cap[1]),
                        DAIKON_ENTER => {
                            ret.add_entity(&cap[1]);
                            inftype = InfType::PreCondition
                        }
                        DAIKON_EXIT => inftype = InfType::PostCondition,
                        _ => println!("Skipping (line {}): {}", i + 1, &cap[2]),
                    };

                    curr_entity = &cap[1];
                }

                continue;
            }

            // invariant definition
            let implies = Regex::new(r"==>").unwrap();
            if implies.is_match(line) {
                println!("Skipping unsupported syntax: {}", line);
                continue;
            }

            let parts = Regex::new(r"^(\S+) ([!=<>]+) (\S+)$").unwrap();
            if parts.is_match(line) {
                for cap in parts.captures_iter(line) {
                    let inv = match &cap[3] {
                        DAIKON_NULL => match &cap[2] {
                            DAIKON_EQ => Invariant::Null {
                                exp: cap[1].to_string(),
                            },
                            DAIKON_NOTEQ => Invariant::NotNull {
                                exp: cap[1].to_string(),
                            },
                            _ => panic!("Invalid operator on NULL: {}", &cap[2]),
                        },
                        _ => Invariant::Comparison {
                            lhs: cap[1].to_string(),
                            operator: cap[2].to_string(),
                            rhs: cap[3].to_string(),
                        },
                    };

                    ret.add_inv(curr_entity, inv, inftype);
                }
            } else {
                println!("Warning: skipping line {}: {}", i + 1, line);
            }
        }

        ret
    }

    pub fn from_file(path: &str) -> Result<InvariantList, io::Error> {
        let mut file = fs::File::open(path)?;
        let mut contents = String::new();

        file.read_to_string(&mut contents)?;
        Ok(Invariants::parse(contents.as_str()))
    }
}
