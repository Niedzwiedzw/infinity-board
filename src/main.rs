use clap::Parser;
use clap::ValueEnum;
use derive_more::Constructor;
use itertools::Itertools;
use std::collections::HashSet;
use std::fmt::Display;
use strum::{EnumCount, EnumIter, IntoEnumIterator};

#[derive(
    Debug, EnumIter, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, EnumCount, Hash, ValueEnum,
)]
pub enum Note {
    C,
    Cs,
    D,
    Ds,
    E,
    F,
    Fs,
    G,
    Gs,
    A,
    As,
    B,
}

impl std::fmt::Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let repr = match self {
            Note::C => "C",
            Note::Cs => "C#",
            Note::D => "D",
            Note::Ds => "D#",
            Note::E => "E",
            Note::F => "F",
            Note::Fs => "F#",
            Note::G => "G",
            Note::Gs => "G#",
            Note::A => "A",
            Note::As => "A#",
            Note::B => "B",
        };
        write!(f, "{}", repr)
    }
}

impl Note {
    fn cycle() -> impl Iterator<Item = Self> {
        Self::iter().cycle()
    }

    fn cycle_from(self) -> impl Iterator<Item = Self> {
        Self::cycle().skip_while(move |i| i != &self)
    }

    fn offset_by(self, offset: i32) -> Self {
        let offset = offset.wrapping_rem_euclid(Self::COUNT as i32);
        self.cycle_from()
            .skip(offset as _)
            .next()
            .expect("this is an infinite stream, come on")
    }
}

#[derive(Debug, Constructor)]
struct GuitarString {
    start: Note,
}

#[derive(Debug)]
struct Guitar {
    strings: Vec<GuitarString>,
    notes_per_string: usize,
}

#[derive(
    Debug, EnumIter, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, EnumCount, Hash, ValueEnum,
)]
pub enum Tuning {
    Fourths,
    ScaleCentered,
}
impl Guitar {
    pub fn from_tuning(
        string_count: usize,
        start: Note,
        notes_per_string: usize,
        tuning: Tuning,
    ) -> Self {
        let strings = match tuning {
            Tuning::Fourths => start
                .cycle_from()
                .step_by(5)
                .take(string_count)
                .map(GuitarString::new)
                .collect(),
            Tuning::ScaleCentered => {
                let intervals: Vec<usize> = vec![4, 4, 4, 4];
                let mut output = vec![start];
                intervals
                    .iter()
                    .cycle()
                    .take(string_count)
                    .for_each(|interval| {
                        let last = output.last().expect("it is not empty").clone();
                        output.push(last.offset_by(*interval as _));
                    });
                output.into_iter().map(GuitarString::new).collect()
            }
        };
        Self {
            strings,
            notes_per_string,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ScaleMode {
    Major,
}

impl ScaleMode {
    pub fn intervals_raw(self) -> Vec<usize> {
        match self {
            ScaleMode::Major => {
                vec![2, 2, 1, 2, 2, 2, 1]
            }
        }
    }
    pub fn intervals(self) -> impl Iterator<Item = usize> {
        self.intervals_raw().into_iter().cycle()
    }
}

#[derive(Debug, Clone, Copy)]
struct Scale {
    start_note: Note,
    mode: ScaleMode,
}

impl Display for Scale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { start_note, mode } = self;
        write!(f, "{start_note} {mode:?}")
    }
}

impl Scale {
    pub fn notes_list(&self) -> Vec<Note> {
        let mut notes = vec![self.start_note];
        let intervals = self.mode.intervals_raw();
        intervals.iter().for_each(|interval| {
            let latest = notes
                .last()
                .cloned()
                .unwrap_or(self.start_note)
                .offset_by(*interval as _);
            notes.push(latest)
        });
        notes
    }

    pub fn notes(&self) -> HashSet<Note> {
        self.notes_list().into_iter().collect()
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long)]
    start_note: Note,
    #[arg(long)]
    mode: ScaleMode,
    #[arg(long)]
    string_count: usize,
    #[arg(long)]
    all_note_names: bool,
    #[arg(long, default_value = "0")]
    frets_start: usize,
    #[arg(long, default_value = "24")]
    frets_end: usize,
    #[arg(long, default_value = "fourths")]
    tuning: Tuning,
}

fn main() {
    let Cli {
        start_note,
        mode,
        string_count,
        all_note_names,
        frets_start,
        frets_end,
        tuning,
    } = Cli::parse();
    let my_tuning = Guitar::from_tuning(string_count, Note::E, frets_end, tuning);
    let scale = Scale { start_note, mode };
    let notes = scale.notes();
    println!("SCALE: {scale}");
    println!(
        "NOTES: {}",
        scale.notes_list().iter().map(|n| n.to_string()).join(", ")
    );
    println!();
    for (num, string) in my_tuning
        .strings
        .iter()
        .enumerate()
        .map(|(i, val)| (i + 1, val))
        .rev()
    {
        print!("{num}({})\t", string.start);
        for note in string
            .start
            .cycle_from()
            .skip(frets_start)
            .take(my_tuning.notes_per_string - frets_start)
        {
            let print_note = |note: &Note| match scale.start_note.eq(note) {
                true => print!("\t\x1b[93m{note}\x1b[0m"),
                false => print!("\t{note}"),
            };
            match notes.contains(&note) {
                true => match all_note_names {
                    true => print_note(&note),
                    false => match note.eq(&scale.start_note) {
                        true => print_note(&note),
                        false => print!("\tO"),
                    },
                },
                false => print!("\t|"),
            }
        }
        print!("\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_offset() {
        assert_eq!(Note::C.offset_by(1), Note::Cs);
        assert_eq!(Note::C.offset_by(-1), Note::B);
    }
}
