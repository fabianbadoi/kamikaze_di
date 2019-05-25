extern crate kamikaze_di;
#[macro_use]
extern crate kamikaze_di_derive;

use std::cell::Cell;
use std::rc::Rc;
use kamikaze_di::{Container, ContainerBuilder, Inject, InjectAsRc, Result, Injector};

const TEXT_RESET: &str = "\x1b[1;0m";
const TEXT_BOLD: &str = "\x1b[1;1m";
const TEXT_ITALIC: &str = "\x1b[1;3m";
const TEXT_COLOR_RED: &str = "\x1b[1;31m";
const TEXT_COLOR_GRAY: &str = "\x1b[1;90m";

#[derive(Clone)]
struct Config {
    lines: Vec<(String, String)>,
    caps_color: String,
    italic_color: String,
    normal_color: String,
}

trait Voice {
    fn say(&self, line: &str) -> String;
}

#[derive(Clone)]
struct Normal {
    color: String,
}

impl Inject for Normal {
    fn resolve(container: &Container) -> Result<Normal> {
        let config: Config = container.inject()?;
        let color = config.normal_color.clone();

        Ok(Normal { color })
    }
}

impl Voice for Normal {
    fn say(&self, line: &str) -> String {
        color(&self.color, line)
    }
}

#[derive(Clone)]
struct Loud {
    color: String,
}

impl Inject for Loud {
    fn resolve(container: &Container) -> Result<Loud> {
        let config: Config = container.inject()?;
        let color = config.caps_color.clone();

        Ok(Loud { color })
    }
}

impl Voice for Loud {
    fn say(&self, line: &str) -> String {
        let line = format!("{}{}{}", TEXT_BOLD, line, TEXT_RESET);
        color(&self.color, &line)
    }
}

#[derive(Clone)]
struct Soft {
    color: String,
}

impl Inject for Soft {
    fn resolve(container: &Container) -> Result<Soft> {
        let config: Config = container.inject()?;
        let color = config.italic_color.clone();

        Ok(Soft { color })
    }
}

impl Voice for Soft {
    fn say(&self, line: &str) -> String {
        let line = format!("{}{}{}", TEXT_ITALIC, line, TEXT_RESET);
        color(&self.color, &line)
    }
}

struct VoiceBox {
    voices: Vec<Box<dyn Voice>>,
    next_voice: Cell<usize>,
}

impl InjectAsRc for VoiceBox {
    fn resolve(container: &Container) -> Result<VoiceBox> {
        let normal: Normal = container.inject()?;
        let loud: Loud = container.inject()?;
        let soft: Soft = container.inject()?;
            
        Ok(VoiceBox {
            voices: vec![
                Box::new(normal),
                Box::new(loud),
                Box::new(soft),
            ],
            next_voice: 0.into(),
        })
    }
}

impl VoiceBox {
    pub fn say(&self, line: &str) -> String {
        let voice = self.next_voice();
        voice.say(line)
    }

    fn next_voice(&self) -> &dyn Voice {
        let voice_index = self.next_voice.get();
        let voice = &*self.voices[voice_index % self.voices.len()];

        self.next_voice.set(voice_index + 1);

        voice
    }
}

#[derive(Clone)]
struct Line(String, String);

#[derive(Inject, Clone)]
struct Jester {
    voice_box: Rc<VoiceBox>,
    lines: Vec<Line>,
}

impl Jester {
    fn perform(&self) {
        for line in &self.lines {
            println!("{}: {}", line.0, self.voice_box.say(&line.1));
        }
    }
}

fn color(color: &str, string: &str) -> String {
    format!("{}{}{}", color, string, TEXT_RESET)
}

fn main() {
    let mut builder = ContainerBuilder::new();
    builder.register(Config {
        lines: vec![
            ("CINNA".to_owned(), "O Caesar,--".to_owned()),

            ("CAESAR".to_owned(), "Hence! wilt thou lift up Olympus?".to_owned()),
            ("DECIUS BRUTUS".to_owned(), "Great Caesar,--".to_owned()),
            ("CAESAR".to_owned(), "Doth not Brutus bootless kneel?".to_owned()),
            ("CASCA".to_owned(), "Speak, hands for me!".to_owned()),
            ("-".to_owned(), "CASCA first, then the other Conspirators and BRUTUS stab CAESAR".to_owned()),
            ("CAESAR".to_owned(), "Et tu, Brute!".to_owned()),
        ],
        normal_color: "".to_owned(),
        italic_color: TEXT_COLOR_GRAY.to_owned(),
        caps_color: TEXT_COLOR_RED.to_owned(),
    }).unwrap();
    builder.register_builder(|container| {
        let config: Config = container.inject().unwrap();
        let lines: Vec<Line> = config.lines
            .iter()
            .map(|l| Line(l.0.clone(), l.1.clone()))
            .collect();

        lines
    }).unwrap();

    let container = builder.build();

    let jester: Jester = container.inject().unwrap();
    jester.perform();
}
