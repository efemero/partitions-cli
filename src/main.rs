// main.rs
//
// Copyright (c) 2025 François Bastien
// All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-or-later
//
// This program is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by the
// Free Software Foundation, either version 3 of the License, or (at your
// option) any later version.
//
// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General
// Public License for more details.
//
// You should have received a copy of the GNU General Public License along
// with this program. If not, see <https://www.gnu.org/licenses/>.

mod types;

use std::process::ExitStatus;

use camino::Utf8PathBuf;
use clap::Parser;
use rayon::prelude::*;
use std::fmt::Write as _;
use std::process::Command;
use walkdir::WalkDir;

use crate::types::{Clef, Format, Instrument, MusicSheet, Tone, Voice};

/// Tool to build and manage music sheets
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
enum Cli {
    Lilypond(LilypondArgs),
}

/// Compile the ly files into pdf
#[derive(clap::Args, Debug)]
#[command(version, about, long_about = None)]
struct LilypondArgs {
    /// Path to the music sources
    #[arg(short, long)]
    limit: Option<usize>,

    /// Path to the music sources
    #[arg(short, long, default_value_t = Utf8PathBuf::from("music") )]
    music_path: Utf8PathBuf,

    /// Only compile the selected title
    #[arg(short, long)]
    title: Option<String>,

    /// Only compile the selected instrument
    #[arg(short, long)]
    instrument: Option<Instrument>,

    /// Only compile the selected voice
    #[arg(short, long)]
    voice: Option<Voice>,

    /// Only compile the selected clef
    #[arg(short, long)]
    clef: Option<Clef>,

    /// Only compile the selected format
    #[arg(short, long)]
    format: Option<Format>,

    /// Only compile the selected tone
    #[arg(short = 'o', long)]
    tone: Option<Tone>,
}

fn main() {
    let cli = Cli::parse();
    match cli {
        Cli::Lilypond(args) => {
            let ly_files: Vec<MusicSheet> = get_ly_files(args);
            compile_lilypond(&ly_files);
        }
    }
}

fn compile_lilypond(music_sheets: &[MusicSheet]) {
    music_sheets.par_iter().for_each(|music_sheet| {
        let output = Command::new("lilypond")
            .arg("-o")
            .arg(&music_sheet.pdf)
            .arg("-fpdf")
            .arg("-dresolution=300")
            .arg("-dpoint-and-click=#f")
            .arg(&music_sheet.source)
            .output();
        let mut description = format!("{} - {}", music_sheet.title, music_sheet.instrument);
        if let Some(tone) = music_sheet.tone {
            description.push(' ');
            description.push_str(&tone.to_string());
        }
        if let Some(voice) = music_sheet.voice {
            description.push(' ');
            description.push_str(&voice.to_string());
        }
        if music_sheet.clef == Clef::ClefFa {
            description.push_str(" (Clef de Fa)");
        }
        let _ = write!(description, " [{}]", music_sheet.format);
        println!(
            "{}: {}",
            description,
            output.map_or(ExitStatus::default(), |o| o.status)
        );
    });
}

fn get_ly_files(args: LilypondArgs) -> Vec<MusicSheet> {
    let files: Vec<MusicSheet> = WalkDir::new(args.music_path)
        .into_iter()
        // map to path
        .filter_map(|e| e.ok().map(walkdir::DirEntry::into_path))
        // filter ly files
        .filter(|p| p.is_file() && p.extension().is_some_and(|s| s == "ly"))
        // take only files if contained in a 'a4' or 'carnet' dir
        .filter(|p| {
            p.components()
                .rev()
                .nth(1)
                .is_some_and(|c| c.as_os_str() == "a4" || c.as_os_str() == "carnet")
        })
        .filter_map(|p| p.try_into().ok())
        .filter_map(|p: Utf8PathBuf| MusicSheet::try_from(p).ok())
        .filter(|m| args.title.as_ref().is_none_or(|t| t == &m.title))
        .filter(|m| args.instrument.as_ref().is_none_or(|i| i == &m.instrument))
        .filter(|m| args.format.as_ref().is_none_or(|f| f == &m.format))
        .filter(|m| args.clef.as_ref().is_none_or(|c| c == &m.clef))
        .filter(|m| {
            args.voice
                .as_ref()
                .is_none_or(|v| Some(v) == m.voice.as_ref())
        })
        .filter(|m| {
            args.tone
                .as_ref()
                .is_none_or(|t| Some(t) == m.tone.as_ref())
        })
        .collect();
    if let Some(limit) = args.limit {
        return files.iter().take(limit).cloned().collect();
    }
    files
}
